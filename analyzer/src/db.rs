//! Wrapper around rocksdb with logic-graph specific features.
extern crate rocksdb;

use crate::context::*;
use crate::estimates::ContextScorer;
use crate::steiner::*;
use crate::world::*;
use anyhow::Result;
use humansize::{SizeFormatter, BINARY};
use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins, Plot};
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use rmp_serde::Serializer;
use rocksdb::{
    perf, BlockBasedOptions, Cache, ColumnFamily, ColumnFamilyDescriptor, CuckooTableOptions, Env,
    IteratorMode, MergeOperands, Options, ReadOptions, WriteBatch, WriteBatchWithTransaction,
    WriteOptions, DB,
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

// We need the following in this wrapper impl:
// 1. The queue db is mainly iterated over, via either
//    getting the minimum-score element (i.e. iterating from start)
//    or running over the whole db (e.g. for statistics). BlockDB is best for this.
// 2. We'll add two LRU cache layers that must outlive the BlockDB,
//    one for uncompressed blocks and the other for compressed blocks.

// We will have the following DBs:
// 1. the queue: (progress, elapsed, seq) -> Ctx
// 2. next: (Ctx, history step) -> (elapsed, Ctx)
// 3. best: Ctx -> (elapsed, history step, prev Ctx)

struct HeapDBOptions {
    opts: Options,
    path: PathBuf,
}

const BEST: &str = "best";

type NextData<T> = (u32, HistoryAlias<T>, T);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct StateData<T, I, S, L, E, A, Wp> {
    // Ordering is important here, since min_merge will sort by serialized bytes.
    elapsed: u32,
    hist: Option<History<I, S, L, E, A, Wp>>,
    prev: Option<T>,
}

type StateDataAlias<T> = StateData<
    T,
    <T as Ctx>::ItemId,
    <<<T as Ctx>::World as World>::Exit as Exit>::SpotId,
    <<<T as Ctx>::World as World>::Location as Location>::LocId,
    <<<T as Ctx>::World as World>::Exit as Exit>::ExitId,
    <<<T as Ctx>::World as World>::Action as Action>::ActionId,
    <<<T as Ctx>::World as World>::Warp as Warp>::WarpId,
>;

pub struct HeapDB<'w, W: World, T: Ctx> {
    scorer: ContextScorer<
        'w,
        W,
        <<W as World>::Exit as Exit>::SpotId,
        <<W as World>::Location as Location>::LocId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    >,
    db: DB,
    nextdb: DB,
    statedb: DB,
    _cache_uncompressed: Cache,
    _cache_cmprsd: Cache,
    _opts: HeapDBOptions,
    _next_opts: HeapDBOptions,
    _state_opts: HeapDBOptions,
    write_opts: WriteOptions,

    max_time: AtomicU32,

    seq: AtomicU32,
    size: AtomicUsize,
    seen: AtomicUsize,
    next: AtomicUsize,
    iskips: AtomicUsize,
    pskips: AtomicUsize,
    dup_iskips: AtomicUsize,
    dup_pskips: AtomicUsize,

    deletes: AtomicUsize,
    delete: AtomicU64,

    bg_deletes: AtomicUsize,

    phantom: PhantomData<T>,
    retrieve_lock: Mutex<()>,
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

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
    W: World<Location = L, Exit = E> + 'w,
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
        opts.set_max_background_jobs(8);
        opts.set_block_based_table_factory(&block_opts);
        let mut env = Env::new().unwrap();
        env.set_low_priority_background_threads(6);
        opts.set_env(&env);

        let next_opts = opts.clone();

        let mut path = p.as_ref().to_owned();
        let mut path2 = path.clone();
        let mut path3 = path.clone();
        path.push("queue");
        path2.push("states");
        path3.push("next");

        // 1 + 2 = 3 GiB roughly for this db
        let _ = DB::destroy(&opts, &path);
        let db = DB::open(&opts, &path)?;
        let _ = DB::destroy(&opts, &path3);
        let nextdb = DB::open(&opts, &path3)?;

        let mut cuckoo_opts = CuckooTableOptions::default();
        cuckoo_opts.set_hash_ratio(0.75);
        cuckoo_opts.set_use_module_hash(false);
        opts2.set_allow_mmap_reads(true);
        opts2.set_allow_mmap_writes(true);
        opts2.set_compression_type(rocksdb::DBCompressionType::None);
        opts2.set_cuckoo_table_factory(&cuckoo_opts);
        opts2.set_merge_operator_associative("min", min_merge);

        let cf_opts = opts2.clone();
        opts2.set_memtable_whole_key_filtering(true);
        opts2.create_missing_column_families(true);

        let bestcf = ColumnFamilyDescriptor::new(BEST, cf_opts);

        // 1 GiB write buffers + 4 GiB row cache = 5GiB ?
        let _ = DB::destroy(&opts2, &path2);
        let statedb = DB::open_cf_descriptors(&opts2, &path2, vec![bestcf])?;

        let mut write_opts = WriteOptions::default();
        write_opts.disable_wal(true);

        let s = Instant::now();
        let scorer = ContextScorer::shortest_paths(world, startctx, 32_768);
        println!("Built scorer in {:?}", s.elapsed());

        Ok(HeapDB {
            scorer,
            db,
            nextdb,
            statedb,
            _cache_uncompressed: cache,
            _cache_cmprsd: cache2,
            _opts: HeapDBOptions { opts, path },
            _next_opts: HeapDBOptions {
                opts: next_opts,
                path: path3,
            },
            _state_opts: HeapDBOptions {
                opts: opts2,
                path: path2,
            },
            write_opts,
            max_time: initial_max_time.into(),
            seq: 0.into(),
            size: 0.into(),
            seen: 0.into(),
            next: 0.into(),
            iskips: 0.into(),
            pskips: 0.into(),
            dup_iskips: 0.into(),
            dup_pskips: 0.into(),
            deletes: 0.into(),
            delete: 0.into(),
            bg_deletes: 0.into(),
            phantom: PhantomData,
            retrieve_lock: Mutex::new(()),
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

    /// Returns the number of processed states (with children) (tracked separately from the db).
    pub fn processed(&self) -> usize {
        self.next.load(Ordering::Acquire)
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
        self.set_max_time(max_time + (max_time / 1024))
    }

    fn best_cf(&self) -> &ColumnFamily {
        self.statedb.cf_handle(BEST).unwrap()
    }

    /// The key for a ContextWrapper<T> in the queue is:
    /// the progress (4 bytes)
    /// the score (4 bytes),
    /// the elapsed time (4 bytes),
    /// a sequence number (4 bytes)
    fn get_heap_key(&self, el: &ContextWrapper<T>) -> [u8; 16] {
        let mut key: [u8; 16] = [0; 16];
        let progress: u32 = self.progress(el.get()).try_into().unwrap();
        key[0..4].copy_from_slice(&progress.to_be_bytes());
        key[4..8].copy_from_slice(&self.score(el).to_be_bytes());
        key[8..12].copy_from_slice(&el.elapsed().to_be_bytes());
        key[12..16].copy_from_slice(&self.seq.fetch_add(1, Ordering::AcqRel).to_be_bytes());
        key
    }

    fn new_heap_key(&self, old_key: &[u8], old_elapsed: u32, new_elapsed: u32) -> [u8; 16] {
        let old_score = u32::from_be_bytes(old_key[4..8].try_into().unwrap());
        let new_score = old_score - old_elapsed + new_elapsed;
        let mut key: [u8; 16] = [0; 16];
        key[0..4].copy_from_slice(&old_key[0..4]);
        // This works because score is an estimated time (requiring deserialization)
        // plus the actual elapsed time
        key[4..8].copy_from_slice(&new_score.to_be_bytes());
        key[8..12].copy_from_slice(&new_elapsed.to_be_bytes());
        key[12..16].copy_from_slice(&self.seq.fetch_add(1, Ordering::AcqRel).to_be_bytes());
        key
    }

    /// The key for a T (Ctx) in the statedb/nextdb, and the value in the queue db
    /// are all T itself.
    fn serialize_state(el: &T) -> Vec<u8> {
        let mut key = Vec::with_capacity(std::mem::size_of::<T>());
        el.serialize(&mut Serializer::new(&mut key)).unwrap();
        key
    }

    fn get_obj_from_heap_value(buf: &[u8]) -> Result<T, Error> {
        Ok(rmp_serde::from_slice::<T>(buf)?)
    }

    fn serialize_data<V>(v: V) -> Vec<u8>
    where
        V: Serialize,
    {
        let mut val = Vec::with_capacity(std::mem::size_of::<V>());
        v.serialize(&mut Serializer::new(&mut val)).unwrap();
        val
    }

    fn get_obj_from_data<V>(buf: &[u8]) -> Result<V, Error>
    where
        V: for<'de> Deserialize<'de>,
    {
        Ok(rmp_serde::from_slice::<V>(buf)?)
    }

    fn get_queue_entry_wrapper(&self, value: &[u8]) -> Result<ContextWrapper<T>, Error> {
        let ctx = Self::get_obj_from_heap_value(value)?;
        let sd = self
            .get_deserialize_state_data(value)?
            .expect("Got unrecognized state from db!");
        Ok(ContextWrapper::with_elapsed(ctx, sd.elapsed))
    }

    fn get_deserialize_state_data(&self, key: &[u8]) -> Result<Option<StateDataAlias<T>>, Error> {
        match self.statedb.get_pinned_cf(self.best_cf(), key)? {
            Some(slice) => Ok(Some(Self::get_obj_from_data(&slice)?)),
            None => Ok(None),
        }
    }

    fn get_state_values<'a, I>(
        &self,
        cf: &ColumnFamily,
        state_keys: I,
    ) -> Result<Vec<Option<StateDataAlias<T>>>, Error>
    where
        I: Iterator<Item = &'a Vec<u8>>,
    {
        let results = self
            .statedb
            .multi_get_cf(state_keys.into_iter().map(|k| (cf, k)));

        let parsed: Vec<_> = results
            .into_iter()
            .map(|res| match res {
                Err(e) => Err(e.to_string()),
                Ok(None) => Ok(None),
                Ok(Some(slice)) => Ok(Some(Self::get_obj_from_data(&slice).unwrap())),
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

    fn serialize_next_data(next_entries: Vec<NextData<T>>) -> Vec<u8> {
        Self::serialize_data(next_entries)
    }

    fn deserialize_next_data(value: &[u8]) -> Result<Vec<NextData<T>>, Error> {
        Self::get_obj_from_data(value)
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
    pub fn score<R>(&self, el: &R) -> u32
    where
        R: Wrapper<T>,
    {
        el.elapsed() + self.estimated_remaining_time(el.get())
    }

    /// Pushes an element into the db.
    /// If the element's elapsed time is greater than the allowed maximum,
    /// or, if the state has been previously processed or previously seen
    /// with an equal or lower elapsed time, does nothing.
    pub fn push(&self, mut el: ContextWrapper<T>, prev: &Option<T>) -> Result<(), Error> {
        let max_time = self.max_time();
        // Records the history in the statedb, even if over time.
        if !self.record_one(&mut el, prev)? {
            return Ok(());
        }
        if el.elapsed() > max_time || self.score(&el) > max_time {
            self.iskips.fetch_add(1, Ordering::Release);
            return Ok(());
        }
        let key = self.get_heap_key(&el);
        let val = Self::serialize_state(el.get());
        self.db.put_opt(key, val, &self.write_opts)?;
        self.size.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn push_from_queue(&self, el: ContextWrapper<T>) -> Result<(), Error> {
        let key = self.get_heap_key(&el);
        let val = Self::serialize_state(el.get());
        self.db.put_opt(key, val, &self.write_opts)?;
        self.size.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn pop(&self, start_progress: usize) -> anyhow::Result<Option<ContextWrapper<T>>> {
        let _retrieve_lock = self.retrieve_lock.lock().unwrap();
        let mut tail_opts = ReadOptions::default();
        tail_opts.set_tailing(true);
        tail_opts.set_iterate_lower_bound(
            <usize as TryInto<u32>>::try_into(start_progress)
                .unwrap()
                .to_be_bytes(),
        );
        let iter = self.db.iterator_opt(IteratorMode::Start, tail_opts);
        for item in iter {
            let (key, value) = item?;
            let ndeletes = self.deletes.fetch_add(1, Ordering::Acquire) + 1;

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

            let el = self.get_queue_entry_wrapper(&value)?;
            if el.elapsed() > self.max_time() {
                self.pskips.fetch_add(1, Ordering::Release);
                continue;
            }

            if self.remember_processed_raw(&value)? {
                self.dup_pskips.fetch_add(1, Ordering::Release);
                continue;
            }
            // We don't need to check the elapsed time against statedb,
            // because that's where the elapsed time came from
            return Ok(Some(el));
        }

        Ok(None)
    }

    pub fn extend_from_queue<I>(&self, iter: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = ContextWrapper<T>>,
    {
        let mut batch = WriteBatchWithTransaction::<false>::default();
        let max_time = self.max_time();
        let mut skips = 0;
        let mut dups = 0;

        for el in iter {
            if el.elapsed() > max_time || self.score(&el) > max_time {
                skips += 1;
                continue;
            }

            let val = Self::serialize_state(el.get());

            if self.remember_processed_raw(&val).unwrap() {
                dups += 1;
                continue;
            }

            let key = self.get_heap_key(&el);
            batch.put(key, val);
        }
        let new = batch.len();
        self.db.write_opt(batch, &self.write_opts)?;

        self.pskips.fetch_add(skips, Ordering::Release);
        self.dup_pskips.fetch_add(dups, Ordering::Release);
        self.size.fetch_add(new, Ordering::Release);

        Ok(())
    }

    /// Retrieves up to `count` elements from the database, removing them.
    pub fn retrieve(
        &self,
        start_progress: usize,
        count: usize,
    ) -> Result<Vec<ContextWrapper<T>>, Error> {
        let _retrieve_lock = self.retrieve_lock.lock().unwrap();
        let mut res = Vec::with_capacity(count);
        let mut tail_opts = ReadOptions::default();
        tail_opts.set_tailing(true);
        tail_opts.set_iterate_lower_bound(
            <usize as TryInto<u32>>::try_into(start_progress)
                .unwrap()
                .to_be_bytes(),
        );
        let mut iter = self.db.iterator_opt(IteratorMode::Start, tail_opts);

        let mut batch = WriteBatchWithTransaction::<false>::default();

        let mut pops = 1;
        let mut pskips = 0;
        let mut dup_pskips = 0;

        let (key, value) = match iter.next() {
            None => return Ok(Vec::new()),
            Some(el) => el?,
        };
        batch.delete(key);

        let el = self.get_queue_entry_wrapper(&value)?;
        let max_time = self.max_time();
        if el.elapsed() > max_time || self.score(&el) > max_time {
            pskips += 1;
        } else {
            res.push(el);
        }

        let start = Instant::now();
        'outer: while res.len() < count {
            loop {
                if let Some(item) = iter.next() {
                    let (key, value) = item.unwrap();
                    batch.delete(key);
                    pops += 1;

                    let el = self.get_queue_entry_wrapper(&value)?;
                    let max_time = self.max_time();
                    if el.elapsed() > max_time || self.score(&el) > max_time {
                        pskips += 1;
                        continue;
                    }
                    if self.remember_processed_raw(&value)? {
                        dup_pskips += 1;
                        continue;
                    }

                    res.push(el);
                    if res.len() == count {
                        break 'outer;
                    }
                } else {
                    break 'outer;
                }
            }
        }
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

    fn remember_processed_raw(&self, key: &[u8]) -> Result<bool, Error> {
        Ok(self.nextdb.key_may_exist(key) && self.nextdb.get_pinned(key)?.is_some())
    }

    /// Checks whether the given Ctx was already processed into its next states.
    pub fn remember_processed(&self, el: &T) -> Result<bool, Error> {
        let next_key = Self::serialize_state(el);
        self.remember_processed_raw(&next_key)
    }

    pub fn count_duplicate(&self) {
        self.dup_pskips.fetch_add(1, Ordering::Release);
    }

    pub fn get_best_elapsed(&self, el: &T) -> Result<u32, Error> {
        let state_key = Self::serialize_state(el);
        let sd = self.get_deserialize_state_data(&state_key)?.unwrap();
        Ok(sd.elapsed)
    }

    fn record_one_batch<const TR: bool>(
        &self,
        state_key: Vec<u8>,
        el: &mut ContextWrapper<T>,
        prev: &Option<T>,
        next_entries: &mut Vec<NextData<T>>,
        state_batch: &mut rocksdb::WriteBatchWithTransaction<TR>,
    ) {
        let hist = el.remove_history();
        assert!(hist.len() == 1);
        let hist = hist.last().copied();
        if let Some((h, dur)) = hist {
            next_entries.push((dur, h, el.get().clone()));
        }
        state_batch.merge_cf(
            self.best_cf(),
            state_key,
            Self::serialize_data(StateData {
                elapsed: el.elapsed(),
                hist: hist.map(|p| p.0),
                prev: prev.clone(),
            }),
        );
    }

    /// Stores the underlying Ctx in the seen db with the best known elapsed time and
    /// its related history is also stored in the db,
    /// and returns whether this context had that best time.
    /// The Wrapper object is modified to reference the stored history.
    /// A `false` value means the state should be skipped.
    pub fn record_one(&self, el: &mut ContextWrapper<T>, prev: &Option<T>) -> Result<bool, Error> {
        let state_key = Self::serialize_state(el.get());
        let is_new =
            // TODO: Maybe we can make this deserialization cheaper as we only need one field?
            if let Some(StateData { elapsed, .. }) = self.get_deserialize_state_data(&state_key)? {
                // This is a new state being pushed, as it has new history, hence we skip if equal.
                if elapsed <= el.elapsed() {
                    self.dup_iskips.fetch_add(1, Ordering::Release);
                    return Ok(false);
                }
                false
            } else {
                true
            };
        // In every other case (no such state, or we do better than that state),
        // we will rewrite the data.

        // We should also check the StateData for whether we even need to do this
        let mut next_entries = Vec::new();
        let mut state_batch = WriteBatch::default();
        self.record_one_batch(state_key, el, prev, &mut next_entries, &mut state_batch);
        if let Some(p) = prev {
            self.nextdb
                .put_opt(
                    Self::serialize_state(p),
                    Self::serialize_next_data(next_entries),
                    &self.write_opts,
                )
                .unwrap();
            self.next.fetch_add(1, Ordering::Release);
        }
        self.statedb
            .write_opt(state_batch, &self.write_opts)
            .unwrap();
        if is_new {
            self.seen.fetch_add(1, Ordering::Release);
        }
        Ok(true)
    }

    /// Stores the underlying Ctx entries in the state db with their respective
    /// best known elapsed times and preceding states,
    /// and returns whether each context had that best time.
    /// Wrapper objects are modified to reference the stored history.
    /// A `false` value for a context means the state should be skipped.
    pub fn record_many(
        &self,
        vec: &mut Vec<ContextWrapper<T>>,
        prev: &Option<T>,
    ) -> Result<Vec<bool>, Error> {
        let mut next_entries = Vec::new();
        let mut state_batch = WriteBatchWithTransaction::<false>::default();
        let mut results = Vec::with_capacity(vec.len());
        let mut dups = 0;
        let mut new_seen = 0;
        let cf = self.best_cf();

        let seeing: Vec<_> = vec
            .iter()
            .map(|el| Self::serialize_state(el.get()))
            .collect();

        let seen_values = self.get_state_values(cf, seeing.iter())?;

        for ((el, state_key), seen_val) in vec
            .iter_mut()
            .zip(seeing.into_iter())
            .zip(seen_values.into_iter())
        {
            if let Some(StateData { elapsed, .. }) = seen_val {
                // This is a new state being pushed, as it has new history, hence we skip if equal.
                if elapsed <= el.elapsed() {
                    results.push(false);
                    dups += 1;
                    continue;
                }
            } else {
                new_seen += 1;
            }
            // In every other case (no such state, or we do better than that state),
            // we will rewrite the data.
            self.record_one_batch(state_key, el, prev, &mut next_entries, &mut state_batch);
            results.push(true);
        }

        if let Some(p) = prev {
            self.nextdb
                .put_opt(
                    Self::serialize_state(p),
                    Self::serialize_next_data(next_entries),
                    &self.write_opts,
                )
                .unwrap();
            self.next.fetch_add(1, Ordering::Release);
        }

        self.statedb
            .write_opt(state_batch, &self.write_opts)
            .unwrap();
        self.dup_iskips.fetch_add(dups, Ordering::Release);
        self.seen.fetch_add(new_seen, Ordering::Release);
        if prev.is_some() {
            self.next.fetch_add(1, Ordering::Release);
        }
        Ok(results)
    }

    pub fn cleanup(&self, batch_size: usize, exit_signal: &AtomicBool) -> Result<(), Error> {
        let mut tail_opts = ReadOptions::default();
        tail_opts.set_tailing(true);
        let mut iter = self.db.iterator_opt(IteratorMode::Start, tail_opts);

        while !exit_signal.load(Ordering::Acquire) {
            let mut batch = WriteBatchWithTransaction::<false>::default();
            let mut pskips = 0;
            let mut dup_pskips = 0;
            let mut rescores = 0;
            let mut count = batch_size;
            let mut compact = false;
            let _retrieve_lock = self.retrieve_lock.lock().unwrap();

            while count > 0 {
                if let Some(item) = iter.next() {
                    let (key, value) = item.unwrap();
                    count -= 1;

                    if self.remember_processed_raw(&value)? {
                        batch.delete(key);
                        dup_pskips += 1;
                        continue;
                    }

                    let StateData { elapsed, .. } = self
                        .get_deserialize_state_data(&value)?
                        .expect("Bg thread found unrecognized state in the db!");
                    let max_time = self.max_time();
                    if elapsed > max_time {
                        batch.delete(key);
                        pskips += 1;
                        continue;
                    }

                    let known = u32::from_be_bytes(key[4..8].as_ref().try_into().unwrap());
                    if known > elapsed {
                        batch.delete(&key);
                        let new_key = self.new_heap_key(&key, known, elapsed);
                        batch.put(new_key, value);
                        rescores += 1;
                    }
                } else {
                    compact = true;
                    break;
                }
            }
            self.db.write_opt(batch, &self.write_opts).unwrap();
            drop(_retrieve_lock);
            self.pskips.fetch_add(pskips, Ordering::Release);
            self.dup_pskips.fetch_add(dup_pskips, Ordering::Release);
            self.bg_deletes
                .fetch_add(pskips + dup_pskips, Ordering::Release);
            self.size.fetch_sub(pskips + dup_pskips, Ordering::Release);
            if pskips > 0 || dup_pskips > 0 || rescores > 0 {
                println!(
                    "Background thread: {} expired, {} duplicate, {} rescored",
                    pskips, dup_pskips, rescores
                );
            }
            if compact {
                let start = Instant::now();
                self.db.compact_range(None::<&[u8]>, None::<&[u8]>);
                println!("Bg thread compacting took {:?}", start.elapsed());
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn get_history(&self, ctx: &T) -> Result<Vec<HistoryAlias<T>>, Error> {
        let mut vec = Vec::new();
        let mut state_key = Self::serialize_state(ctx);
        loop {
            if let Some(StateData { hist, prev, .. }) =
                self.get_deserialize_state_data(&state_key)?
            {
                if let Some(ctx) = prev {
                    state_key = Self::serialize_state(&ctx);
                    vec.push(hist.unwrap());
                } else {
                    break;
                }
            } else {
                return Err(Error {
                    message: format!("Could not find state entry for {:?}", ctx),
                });
            }
        }
        vec.reverse();
        Ok(vec)
    }

    pub fn get_history_ctx(&self, ctx: &ContextWrapper<T>) -> Result<Vec<HistoryAlias<T>>, Error> {
        match self.get_history(ctx.get()) {
            Ok(mut vec) => {
                vec.extend(ctx.recent_history().iter().map(|p| p.0));
                Ok(vec)
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_last_history_step(
        &self,
        ctx: &ContextWrapper<T>,
    ) -> Result<Option<HistoryAlias<T>>, Error> {
        if let Some(h) = ctx.recent_history().last() {
            Ok(Some(h.0))
        } else {
            Ok(self
                .get_deserialize_state_data(&Self::serialize_state(ctx.get()))?
                .map(|sd| sd.hist)
                .flatten())
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
            let el = self.get_queue_entry_wrapper(&value)?;
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
