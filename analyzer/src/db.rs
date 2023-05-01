//! Wrapper around rocksdb with logic-graph specific features.
extern crate rocksdb;

use crate::context::*;
use crate::estimates::ContextScorer;
use crate::steiner::*;
use crate::world::*;
use humansize::{SizeFormatter, BINARY};
use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins, Plot};
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use rmp_serde::Serializer;
use rocksdb::Env;
use rocksdb::{
    perf, BlockBasedOptions, Cache, ColumnFamily, ColumnFamilyDescriptor, CuckooTableOptions,
    IteratorMode, MergeOperands, Options, ReadOptions, WriteBatchWithTransaction, WriteOptions, DB,
};
use serde::Serialize;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

// We need the following in this wrapper impl:
// 1. The contextwrapper db is mainly iterated over, via either
//    getting the minimum-score element (i.e. iterating from start)
//    or running over the whole db (e.g. for statistics). BlockDB is best for this.
// 2. We'll add two LRU cache layers that must outlive the BlockDB,
//    one for uncompressed blocks and the other for compressed blocks.
// 3. The best-seen version of a context is mainly point lookups,
//    which is better served by CuckooTable. This will be a Ctx -> u32
//    map, which fulfills the fixed-length key and value limitations.

struct HeapDBOptions {
    opts: Options,
    path: PathBuf,
}

pub struct HeapDB<'w, W: World, T> {
    scorer: ContextScorer<
        'w,
        W,
        <<W as World>::Exit as Exit>::SpotId,
        <<W as World>::Location as Location>::LocId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    >,
    db: DB,
    statedb: DB,
    _cache_uncompressed: Cache,
    _cache_cmprsd: Cache,
    _opts: HeapDBOptions,
    _state_opts: HeapDBOptions,
    write_opts: WriteOptions,

    max_time: AtomicU32,

    seq: AtomicU64,
    size: AtomicUsize,
    seen: AtomicUsize,
    iskips: AtomicUsize,
    pskips: AtomicUsize,
    dup_iskips: AtomicUsize,
    dup_pskips: AtomicUsize,

    deletes: AtomicUsize,
    delete: AtomicU64,

    retrieve_lock: Mutex<()>,
    bg_deletes: AtomicUsize,

    phantom: PhantomData<T>,
}

// Final cleanup, done in a separate struct here to ensure it's done
// after the db is dropped.
impl Drop for HeapDBOptions {
    fn drop(&mut self) {
        let _ = DB::destroy(&self.opts, &self.path);
    }
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl From<rocksdb::Error> for Error {
    fn from(value: rocksdb::Error) -> Self {
        Error {
            message: value.into(),
        }
    }
}

impl From<rmp_serde::decode::Error> for Error {
    fn from(value: rmp_serde::decode::Error) -> Self {
        Error {
            message: format!("{:?}", value),
        }
    }
}

impl From<Error> for String {
    fn from(value: Error) -> Self {
        value.message
    }
}

fn min_merge(
    _new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    if let Some(res) = operands.iter().min() {
        if let Some(v) = existing_val {
            if res < v {
                Some(res.into())
            } else {
                Some(v.into())
            }
        } else {
            Some(res.into())
        }
    } else {
        existing_val.map(|v| v.into())
    }
}

const MB: usize = 1 << 20;
const GB: usize = 1 << 30;

impl<'w, W, T, L, E> HeapDB<'w, W, T>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = E::Currency>,
{
    pub fn open<P>(
        p: P,
        initial_max_time: u32,
        world: &'w W,
        startctx: &T,
    ) -> Result<HeapDB<'w, W, T>, String>
    where
        P: AsRef<Path>,
    {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        // For now, the db should be deleted.
        opts.set_error_if_exists(true);
        // change compression options?
        // 4 write buffers at 256 MiB = 1 GiB
        opts.set_write_buffer_size(256 * MB);
        opts.set_max_write_buffer_number(4);
        // use half the logical cores, clamp between 2 and 32
        opts.increase_parallelism(std::cmp::max(
            2,
            std::cmp::min(num_cpus::get() / 2, 32).try_into().unwrap(),
        ));

        let mut opts2 = opts.clone();

        let mut block_opts = BlockBasedOptions::default();
        // blockdb caches = 2 GiB
        let cache = Cache::new_lru_cache(GB)?;
        let cache2 = Cache::new_lru_cache(GB)?;
        block_opts.set_block_cache(&cache);
        block_opts.set_block_cache_compressed(&cache2);
        block_opts.set_block_size(1024);
        opts.set_max_background_jobs(16);
        opts.set_block_based_table_factory(&block_opts);
        let mut env = Env::new().unwrap();
        env.set_low_priority_background_threads(12);
        opts.set_env(&env);

        let mut path = p.as_ref().to_owned();
        let mut path2 = path.clone();
        path.push("queue");
        path2.push("states");

        // 1 + 2 = 3 GiB roughly for this db
        let _ = DB::destroy(&opts, &path);
        let db = DB::open(&opts, &path)?;

        let mut cuckoo_opts = CuckooTableOptions::default();
        cuckoo_opts.set_hash_ratio(0.75);
        cuckoo_opts.set_use_module_hash(false);
        opts2.set_cuckoo_table_factory(&cuckoo_opts);
        opts2.set_merge_operator_associative("min", min_merge);
        opts2.set_compression_type(rocksdb::DBCompressionType::None);
        opts2.set_allow_mmap_reads(true);
        opts2.set_allow_mmap_writes(true);

        let cf_opts = opts2.clone();
        opts2.set_memtable_whole_key_filtering(true);
        opts2.create_missing_column_families(true);

        let seencf = ColumnFamilyDescriptor::new("seen", cf_opts.clone());
        let scorecf = ColumnFamilyDescriptor::new("score", cf_opts);

        // 1 GiB write buffers + 4 GiB row cache = 5GiB for this one?
        let _ = DB::destroy(&opts2, &path2);
        let statedb = DB::open_cf_descriptors(&opts2, &path2, vec![seencf, scorecf])?;

        let mut write_opts = WriteOptions::default();
        write_opts.disable_wal(true);

        let s = Instant::now();
        let scorer = ContextScorer::shortest_paths(world, startctx);
        println!("Built scorer in {:?}", s.elapsed());

        Ok(HeapDB {
            scorer,
            db,
            statedb,
            _cache_uncompressed: cache,
            _cache_cmprsd: cache2,
            _opts: HeapDBOptions { opts, path },
            _state_opts: HeapDBOptions {
                opts: opts2,
                path: path2,
            },
            write_opts,
            max_time: initial_max_time.into(),
            seq: 0.into(),
            size: 0.into(),
            seen: 0.into(),
            iskips: 0.into(),
            pskips: 0.into(),
            dup_iskips: 0.into(),
            dup_pskips: 0.into(),
            deletes: 0.into(),
            delete: 0.into(),
            retrieve_lock: Mutex::new(()),
            bg_deletes: 0.into(),
            phantom: PhantomData,
        })
    }

    pub fn scorer(
        &self,
    ) -> &ContextScorer<
        W,
        <<W as World>::Exit as Exit>::SpotId,
        <<W as World>::Location as Location>::LocId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    > {
        &self.scorer
    }

    /// Returns the number of elements in the heap (tracked separately from the db).
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Acquire)
    }

    pub fn is_empty(&self) -> bool {
        self.size.load(Ordering::Acquire) == 0
    }

    /// Returns the number of unique states we've seen so far (tracked separately from the db).
    pub fn seen(&self) -> usize {
        self.seen.load(Ordering::Acquire)
    }

    /// Returns the number of unique states we've estimated remaining time for.
    /// Winning states aren't counted in this.
    pub fn estimates(&self) -> usize {
        self.scorer.estimates()
    }

    /// Returns the number of cache hits for estimated remaining time.
    /// Winning states aren't counted in this.
    pub fn cached_estimates(&self) -> usize {
        self.scorer.cached_estimates()
    }

    pub fn background_deletes(&self) -> usize {
        self.bg_deletes.load(Ordering::Acquire)
    }

    /// Returns details about the number of states we've skipped (tracked separately from the db).
    /// Specifically:
    ///   1) states not added (on push) to the db due to exceeding max time,
    ///   2) states not returned (on pop) from the db due to exceeding max time,
    ///   3) states not added (on push) to the db due to being duplicates with worse times,
    ///   4) states not returned (on pop) from the db due to being duplicates with worse times.
    pub fn skip_stats(&self) -> (usize, usize, usize, usize) {
        (
            self.iskips.load(Ordering::Acquire),
            self.pskips.load(Ordering::Acquire),
            self.dup_iskips.load(Ordering::Acquire),
            self.dup_pskips.load(Ordering::Acquire),
        )
    }

    pub fn max_time(&self) -> u32 {
        self.max_time.load(Ordering::Acquire)
    }

    pub fn set_max_time(&self, max_time: u32) {
        self.max_time.fetch_min(max_time, Ordering::Release);
    }

    pub fn set_lenient_max_time(&self, max_time: u32) {
        self.set_max_time(max_time + (max_time / 128))
    }

    fn seen_cf(&self) -> &ColumnFamily {
        self.statedb.cf_handle("seen").unwrap()
    }

    /// The key for a ContextWrapper<T> in the heap is:
    /// the score (4 bytes),
    /// elapsed time (4 bytes),
    /// a sequence number (8 bytes)
    fn get_heap_key(&self, el: &ContextWrapper<T>) -> [u8; 16] {
        let mut key: [u8; 16] = [0; 16];
        key[0..4].copy_from_slice(&self.score(el).to_be_bytes());
        key[4..8].copy_from_slice(&el.elapsed().to_be_bytes());
        key[8..16].copy_from_slice(&self.seq.fetch_add(1, Ordering::AcqRel).to_be_bytes());
        key
    }

    /// The key for a T (Ctx) in the statedb is... itself!
    fn get_state_key(el: &T) -> Vec<u8> {
        let mut key = Vec::with_capacity(std::mem::size_of::<T>());
        el.serialize(&mut Serializer::new(&mut key)).unwrap();
        key
    }

    /// The value for a ContextWrapper<T> in the heap is... itself!
    /// Unfortunately this is serializing the T (Ctx) a second time if we already got the state key.
    fn get_heap_value(el: &ContextWrapper<T>) -> Vec<u8> {
        let mut val = Vec::new();
        el.serialize(&mut Serializer::new(&mut val)).unwrap();
        val
    }

    fn get_obj_from_heap_value(buf: &[u8]) -> Result<ContextWrapper<T>, Error> {
        Ok(rmp_serde::from_slice::<ContextWrapper<T>>(buf)?)
    }

    fn get_state_value(&self, cf: &ColumnFamily, state_key: &[u8]) -> Result<Option<u32>, Error> {
        match self.statedb.get_pinned_cf(cf, state_key)? {
            Some(slice) => {
                if slice.len() != 4 {
                    return Err(Error {
                        message: format!("Invalid seen elapsed time length: {}", slice.len()),
                    });
                }
                Ok(Some(u32::from_be_bytes(slice.as_ref().try_into().unwrap())))
            }
            None => Ok(None),
        }
    }

    fn get_seen_value(&self, state_key: &[u8]) -> Result<Option<u32>, Error> {
        self.get_state_value(self.seen_cf(), state_key)
    }

    fn get_state_values<'a, I>(
        &self,
        cf: &ColumnFamily,
        state_keys: I,
    ) -> Result<Vec<Option<u32>>, Error>
    where
        I: Iterator<Item = &'a Vec<u8>>,
    {
        let results = self
            .statedb
            .multi_get_cf(state_keys.into_iter().map(|k| (cf, k)));

        let parsed: Vec<Result<Option<u32>, String>> = results
            .into_iter()
            .map(|res| match res {
                Err(e) => Err(e.to_string()),
                Ok(None) => Ok(None),
                Ok(Some(slice)) => {
                    if slice.len() != 4 {
                        Err(format!("Invalid seen elapsed time length: {}", slice.len()))
                    } else {
                        Ok(Some(u32::from_be_bytes(slice.try_into().unwrap())))
                    }
                }
            })
            .collect();

        let error: Vec<String> = parsed
            .iter()
            .filter_map(|res| match res {
                Err(s) => Some(s.to_string()),
                Ok(_) => None,
            })
            .collect();
        if !error.is_empty() {
            Err(Error {
                message: error.join("; "),
            })
        } else {
            Ok(parsed.into_iter().map(|res| res.unwrap()).collect())
        }
    }

    fn get_seen_values<'a, I>(&self, state_keys: I) -> Result<Vec<Option<u32>>, Error>
    where
        I: Iterator<Item = &'a Vec<u8>>,
    {
        self.get_state_values(self.seen_cf(), state_keys)
    }

    /// Estimates the remaining time to the goal.
    pub fn estimated_remaining_time(&self, ctx: &T) -> u32 {
        self.scorer.estimate_remaining_time(ctx).try_into().unwrap()
    }

    /// Returns a number usable as a relative measure of progress.
    /// The number isn't normalized so don't rely on it as fully-ordered,
    /// e.g. two different routes may win at different progress measures.
    pub fn progress(&self, ctx: &T) -> usize {
        self.scorer.required_visits(ctx)
    }

    pub fn estimate_time_to_get(
        &self,
        ctx: &T,
        required: Vec<<<W as World>::Location as Location>::LocId>,
    ) -> u32 {
        self.scorer
            .estimate_time_to_get(ctx, required)
            .try_into()
            .unwrap()
    }

    /// Scores a state based on its elapsed time and its estimated time to the goal.
    /// Recursively estimates time to the goal based on the closest objective item remaining,
    /// and stores the information in the db.
    pub fn score(&self, el: &ContextWrapper<T>) -> u32 {
        el.elapsed() + self.estimated_remaining_time(el.get())
    }

    /// Pushes an element into the heap.
    /// If the element's elapsed time is greater than the allowed maximum,
    /// or, the state has been previously seen with an equal or lower elapsed time, does nothing.
    pub fn push(&self, el: ContextWrapper<T>) -> Result<(), Error> {
        let max_time = self.max_time();
        if el.elapsed() > max_time || self.score(&el) > max_time {
            self.iskips.fetch_add(1, Ordering::Release);
            return Ok(());
        }
        if !self.remember_push(&el)? {
            return Ok(());
        }
        let key = self.get_heap_key(&el);
        let val = Self::get_heap_value(&el);
        self.db.put_opt(key, val, &self.write_opts)?;
        self.size.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn pop(&self, score_hint: Option<u32>) -> Result<Option<ContextWrapper<T>>, Error> {
        let _lock = self.retrieve_lock.lock().unwrap();
        let mut tail_opts = ReadOptions::default();
        tail_opts.set_tailing(true);
        let prefix: [u8; 4];
        let mode = match score_hint {
            None => IteratorMode::Start,
            Some(score) => {
                prefix = score.to_be_bytes();
                IteratorMode::From(&prefix, rocksdb::Direction::Forward)
            }
        };
        let iter = self.db.iterator_opt(mode, tail_opts);
        for item in iter {
            let (key, value) = item?;
            let ndeletes = self.deletes.fetch_add(1, Ordering::Acquire) + 1;
            let mut k = Vec::with_capacity(17);
            (*key).clone_into(&mut k);
            k.push(u8::MAX);

            let raw = u64::from_be_bytes(key[0..8].as_ref().try_into().unwrap()) + 1;
            // Ignore error
            let _ = self.db.delete_opt(key, &self.write_opts);
            self.delete.fetch_max(raw, Ordering::Release);
            self.size.fetch_sub(1, Ordering::Release);

            if ndeletes % 20000 == 0 {
                let start = Instant::now();
                let max_deleted = self.delete.swap(0, Ordering::Acquire);
                self.db
                    .compact_range(None::<&[u8]>, Some(&max_deleted.to_be_bytes()));
                println!("Compacting took {:?}", start.elapsed());
            }

            let el = Self::get_obj_from_heap_value(&value)?;
            if el.elapsed() > self.max_time() {
                self.pskips.fetch_add(1, Ordering::Release);
                continue;
            }

            let seen_key = Self::get_state_key(el.get());
            if let Some(stored) = self.get_seen_value(&seen_key)? {
                if el.elapsed() > stored {
                    self.dup_pskips.fetch_add(1, Ordering::Release);
                    continue;
                }
            }
            return Ok(Some(el));
        }

        Ok(None)
    }

    pub fn extend<I>(&self, iter: I, as_pop: bool) -> Result<(), Error>
    where
        I: IntoIterator<Item = ContextWrapper<T>>,
    {
        let mut batch = WriteBatchWithTransaction::<false>::default();
        let mut seen_batch = WriteBatchWithTransaction::<false>::default();
        let max_time = self.max_time();
        let mut skips = 0;
        let mut dups = 0;

        let cf = self.seen_cf();

        let to_add: Vec<_> = iter
            .into_iter()
            .filter_map(|el| {
                if el.elapsed() > max_time {
                    None
                } else {
                    let seen_key = Self::get_state_key(el.get());
                    Some((el, seen_key))
                }
            })
            .collect();

        let seen_values = self.get_seen_values(to_add.iter().map(|(_, k)| k))?;

        for ((el, seen_key), seen_val) in to_add.into_iter().zip(seen_values.into_iter()) {
            if el.elapsed() > max_time || self.score(&el) > max_time {
                skips += 1;
                continue;
            }

            let should_write = match seen_val {
                Some(stored) => {
                    if stored < el.elapsed() {
                        dups += 1;
                        continue;
                    }
                    stored > el.elapsed()
                }
                None => true,
            };
            // If the value seen is also what we have, we still want to put it into the heap,
            // but we don't have to write the value again as it's a maximum.
            if should_write {
                seen_batch.merge_cf(cf, &seen_key, el.elapsed().to_be_bytes());
            }
            let key = self.get_heap_key(&el);
            let val = Self::get_heap_value(&el);
            batch.put(key, val);
        }
        let new = batch.len();
        let new_seen = seen_batch.len();
        self.db.write_opt(batch, &self.write_opts)?;
        self.statedb.write_opt(seen_batch, &self.write_opts)?;

        self.iskips.fetch_add(skips, Ordering::Release);
        if as_pop {
            self.dup_pskips.fetch_add(dups, Ordering::Release);
        } else {
            self.dup_iskips.fetch_add(dups, Ordering::Release);
        }
        self.size.fetch_add(new, Ordering::Release);
        self.seen.fetch_add(new_seen, Ordering::Release);

        Ok(())
    }

    /// Retrieves up to `count` elements from the database, removing them.
    pub fn retrieve(
        &self,
        start_priority: u32,
        count: usize,
    ) -> Result<Vec<ContextWrapper<T>>, Error> {
        let _lock = self.retrieve_lock.lock().unwrap();
        let mut res = Vec::with_capacity(count);
        let mut tmp = Vec::with_capacity(count);
        let mut tail_opts = ReadOptions::default();
        tail_opts.set_tailing(true);
        tail_opts.set_iterate_lower_bound(start_priority.to_be_bytes());
        let mut iter = self.db.iterator_opt(IteratorMode::Start, tail_opts);

        let mut batch = WriteBatchWithTransaction::<false>::default();

        let mut pops = 1;
        let mut pskips = 0;
        let mut dup_pskips = 0;

        let (key, value) = match iter.next() {
            None => return Ok(Vec::new()),
            Some(el) => el?,
        };
        let mut min = vec![0; 16];
        let mut max = vec![0; 16];
        min.copy_from_slice(&key);
        max.copy_from_slice(&key);
        batch.delete(key);

        let el = Self::get_obj_from_heap_value(&value)?;
        let max_time = self.max_time();
        if el.elapsed() > max_time || self.score(&el) > max_time {
            pskips += 1;
        } else {
            let seen_key = Self::get_state_key(el.get());
            tmp.push((el, seen_key));
        }

        let start = Instant::now();
        let mut done = false;
        while res.len() < count {
            loop {
                if let Some(item) = iter.next() {
                    let (key, value) = item.unwrap();
                    max.copy_from_slice(&key);
                    batch.delete(key);
                    pops += 1;

                    let el = Self::get_obj_from_heap_value(&value)?;
                    let max_time = self.max_time();
                    if el.elapsed() > max_time || self.score(&el) > max_time {
                        pskips += 1;
                        continue;
                    }

                    let seen_key = Self::get_state_key(el.get());
                    tmp.push((el, seen_key));
                    if tmp.len() == count - res.len() {
                        break;
                    }
                } else {
                    done = true;
                    break;
                }
            }

            // Grab all the seen values in one request.
            let seen_values = self.get_seen_values(tmp.iter().map(|(_, k)| k))?;
            res.extend(tmp.into_iter().zip(seen_values.into_iter()).filter_map(
                |((el, _), seen_val)| match seen_val {
                    Some(stored) => {
                        if stored < el.elapsed() {
                            dup_pskips += 1;
                            None
                        } else {
                            Some(el)
                        }
                    }
                    // There should always be a value, but if somehow there isn't, return it for sure.
                    None => Some(el),
                },
            ));
            if done {
                break;
            }
            tmp = Vec::with_capacity(count - res.len());
        }
        max.push(u8::MAX);
        println!(
            "We got {} results in {:?}, having iterated through {} elements",
            res.len(),
            start.elapsed(),
            pops
        );
        // Ignore/assert errors once we start deleting.
        println!("Beginning point deletion of iterated elements...");
        let start = Instant::now();
        self.db.write_opt(batch, &self.write_opts).unwrap();
        println!("Deletes completed in {:?}", start.elapsed());

        self.size.fetch_sub(pops, Ordering::Release);
        self.pskips.fetch_add(pskips, Ordering::Release);
        self.dup_pskips.fetch_add(dup_pskips, Ordering::Release);

        Ok(res)
    }

    fn remember(
        &self,
        el: &ContextWrapper<T>,
        skip_count: &AtomicUsize,
        accept_eq: bool,
    ) -> Result<bool, Error> {
        let seen_key = Self::get_state_key(el.get());

        let should_write = match self.get_seen_value(&seen_key)? {
            Some(stored) => {
                if if accept_eq {
                    stored < el.elapsed()
                } else {
                    stored <= el.elapsed()
                } {
                    skip_count.fetch_add(1, Ordering::Release);
                    return Ok(false);
                }
                stored > el.elapsed()
            }
            None => true,
        };
        // If the value seen is also what we have, we still want to put it into the heap,
        // but we don't have to write the value again as it's a maximum.
        if should_write {
            self.statedb.merge_cf_opt(
                self.seen_cf(),
                &seen_key,
                el.elapsed().to_be_bytes(),
                &self.write_opts,
            )?;
            self.seen.fetch_add(1, Ordering::Release);
        }
        Ok(true)
    }

    /// Stores the underlying Ctx in the seen db with the best known elapsed time,
    /// and returns whether this context had that best time.
    /// A `false` value means the state should be skipped.
    pub fn remember_push(&self, el: &ContextWrapper<T>) -> Result<bool, Error> {
        self.remember(el, &self.dup_iskips, false)
    }

    /// Stores the underlying Ctx in the seen db with the best known elapsed time,
    /// and returns whether this context had that best time.
    /// A `false` value means the state should be skipped.
    pub fn remember_pop(&self, el: &ContextWrapper<T>) -> Result<bool, Error> {
        self.remember(el, &self.dup_pskips, true)
    }

    /// Stores the underlying Ctx entries in the seen db with the respective best known elapsed times,
    /// and returns whether each context had that best time.
    /// A `false` value for a context means the state should be skipped.
    pub fn remember_which(&self, vec: &Vec<ContextWrapper<T>>) -> Result<Vec<bool>, Error> {
        let mut seen_batch = WriteBatchWithTransaction::<false>::default();
        let mut dups = 0;
        let cf = self.seen_cf();

        let seeing: Vec<_> = vec.iter().map(|el| Self::get_state_key(el.get())).collect();

        let seen_values = self.get_seen_values(seeing.iter())?;

        let mut res = Vec::with_capacity(vec.len());

        for ((el, seen_key), seen_val) in vec
            .iter()
            .zip(seeing.into_iter())
            .zip(seen_values.into_iter())
        {
            let should_write = match seen_val {
                Some(stored) => {
                    if stored <= el.elapsed() {
                        dups += 1;
                        res.push(false);
                        continue;
                    } else {
                        res.push(true);
                    }
                    stored > el.elapsed()
                }
                None => {
                    res.push(true);
                    true
                }
            };
            if should_write {
                seen_batch.merge_cf(cf, &seen_key, el.elapsed().to_be_bytes());
            }
        }
        let new_seen = seen_batch.len();
        self.statedb.write_opt(seen_batch, &self.write_opts)?;

        self.dup_iskips.fetch_add(dups, Ordering::Release);
        self.seen.fetch_add(new_seen, Ordering::Release);
        Ok(res)
    }

    pub fn cleanup(&self, batch_size: usize) -> Result<(), Error> {
        let mut tail_opts = ReadOptions::default();
        tail_opts.set_tailing(true);
        let mut iter = self.db.iterator_opt(IteratorMode::Start, tail_opts);

        loop {
            let mut batch = WriteBatchWithTransaction::<false>::default();
            let mut pskips = 0;
            let mut dup_pskips = 0;
            let mut count = batch_size;
            let _lock = self.retrieve_lock.lock().unwrap();

            while count > 0 {
                if let Some(item) = iter.next() {
                    let (key, value) = item.unwrap();
                    count -= 1;

                    let el = Self::get_obj_from_heap_value(&value)?;
                    let max_time = self.max_time();
                    if el.elapsed() > max_time || self.score(&el) > max_time {
                        batch.delete(key);
                        pskips += 1;
                        continue;
                    }

                    let seen_key = Self::get_state_key(el.get());
                    if let Some(stored) = self.get_seen_value(&seen_key)? {
                        if el.elapsed() > stored {
                            batch.delete(key);
                            dup_pskips += 1;
                            continue;
                        }
                    }
                } else {
                    drop(_lock);
                    let start = Instant::now();
                    self.db.compact_range(None::<&[u8]>, None::<&[u8]>);
                    println!("Bg thread compacting took {:?}", start.elapsed());
                    return Ok(());
                }
            }
            self.db.write_opt(batch, &self.write_opts).unwrap();
            drop(_lock);
            self.pskips.fetch_add(pskips, Ordering::Release);
            self.dup_pskips.fetch_add(dup_pskips, Ordering::Release);
            self.bg_deletes
                .fetch_add(pskips + dup_pskips, Ordering::Release);
            if pskips > 0 || dup_pskips > 0 {
                println!(
                    "Background thread deleted {} expired and {} duplicate elements",
                    pskips, dup_pskips
                );
            }
        }
    }

    pub fn print_graphs(&self) -> Result<(), Error> {
        let size = self.size.load(Ordering::Acquire);
        let max_time = self.max_time.load(Ordering::Acquire);
        let mut times: Vec<f64> = Vec::with_capacity(size);
        let mut time_scores: Vec<(f64, f64)> = Vec::with_capacity(size);
        let mut read_opts = ReadOptions::default();
        read_opts.fill_cache(false);
        let iter = self.db.iterator_opt(IteratorMode::Start, read_opts);
        for item in iter {
            let (_, value) = item?;
            let el = Self::get_obj_from_heap_value(&value)?;
            times.push(el.elapsed().into());
            time_scores.push((el.elapsed().into(), self.score(&el).into()));
        }

        let h = Histogram::from_slice(times.as_slice(), HistogramBins::Count(70));
        let v = ContinuousView::new()
            .add(h)
            .x_label("elapsed time")
            .x_range(0., max_time.into());
        println!(
            "Current heap contents:\n{}",
            Page::single(&v).dimensions(90, 10).to_text().unwrap()
        );
        let p = Plot::new(time_scores).point_style(PointStyle::new().marker(PointMarker::Circle));
        let v = ContinuousView::new()
            .add(p)
            .x_label("elapsed time")
            .y_label("score")
            .x_range(0., max_time.into());
        println!(
            "Heap scores by time:\n{}",
            Page::single(&v).dimensions(90, 10).to_text().unwrap()
        );

        Ok(())
    }

    pub fn get_memory_usage_stats(&self) -> Result<String, Error> {
        let dbstats = perf::get_memory_usage_stats(
            Some(&[&self.db]),
            Some(&[&self._cache_cmprsd, &self._cache_uncompressed]),
        )?;
        let statestats = perf::get_memory_usage_stats(Some(&[&self.statedb]), None)?;

        Ok(format!(
            "db: total={}, unflushed={}, readers={}, caches={}\n\
             statedb: total={}, unflushed={}, readers={}, caches={}\n\
             uncompressed={}, compressed={}",
            SizeFormatter::new(dbstats.mem_table_total, BINARY),
            SizeFormatter::new(dbstats.mem_table_unflushed, BINARY),
            SizeFormatter::new(dbstats.mem_table_readers_total, BINARY),
            SizeFormatter::new(dbstats.cache_total, BINARY),
            SizeFormatter::new(statestats.mem_table_total, BINARY),
            SizeFormatter::new(statestats.mem_table_unflushed, BINARY),
            SizeFormatter::new(statestats.mem_table_readers_total, BINARY),
            SizeFormatter::new(statestats.cache_total, BINARY),
            SizeFormatter::new(self._cache_uncompressed.get_usage(), BINARY),
            SizeFormatter::new(self._cache_cmprsd.get_usage(), BINARY),
        ))
    }
}
