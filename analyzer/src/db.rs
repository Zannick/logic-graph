//! Wrapper around rocksdb with logic-graph specific features.
extern crate rocksdb;

use crate::context::*;
use crate::estimates::ContextScorer;
use crate::solutions::Solution;
use crate::solutions::SolutionCollector;
use crate::steiner::*;
use crate::world::*;
use crate::{new_hashmap, CommonHasher};
use anyhow::Result;
use humansize::{SizeFormatter, BINARY};
use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins, Plot};
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use rmp_serde::Serializer;
use rocksdb::{
    perf, BlockBasedOptions, Cache, ColumnFamily, ColumnFamilyDescriptor, Env, IteratorMode,
    MergeOperands, Options, ReadOptions, WriteBatchWithTransaction, WriteOptions, DB,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// We need the following in this wrapper impl:
// 1. The queue db is mainly iterated over, via either
//    getting the minimum-score element (i.e. iterating from start)
//    or running over the whole db (e.g. for statistics). BlockDB is best for this.
// 2. We'll add an LRU cache layer that must outlive the BlockDB.

// We will have the following DBs:
// 1. the queue: (progress, elapsed, seq) -> Ctx
// 2. next: (Ctx, history step) -> (elapsed, Ctx)
// 3. best: Ctx -> (elapsed, history step, prev Ctx)

struct HeapDBOptions {
    opts: Options,
    path: PathBuf,
}

const BEST: &str = "best";
const NEXT: &str = "next";
const TOO_MANY_STEPS: usize = 1024 << 3;

type NextData = (u32, Vec<u8>);

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct StateData<I, S, L, E, A, Wp> {
    // Ordering is important here, since min_merge will sort by serialized bytes.
    elapsed: u32,
    hist: Vec<History<I, S, L, E, A, Wp>>,
    // This is mainly going to be used for performing another lookup, no point
    // in deserializing just to reserialize
    prev: Vec<u8>,
}

type StateDataAlias<T> = StateData<
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
    statedb: DB,
    _cache: Cache,
    _state_cache: Cache,
    _opts: HeapDBOptions,
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

    min_db_estimates: Vec<AtomicU32>,

    bg_deletes: AtomicUsize,

    retrieve_lock: Mutex<()>,
    solutions: Arc<Mutex<SolutionCollector<T>>>,
    world: &'w W,
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
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = E::Currency>,
{
    pub fn open<P>(
        p: P,
        initial_max_time: u32,
        world: &'w W,
        startctx: &T,
        solutions: Arc<Mutex<SolutionCollector<T>>>,
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
        opts.set_target_file_size_base(128 * 1024 * 1024);
        // use half the logical cores, clamp between 2 and 32
        opts.increase_parallelism(std::cmp::max(
            2,
            std::cmp::min(num_cpus::get() / 2, 32).try_into().unwrap(),
        ));
        opts.set_max_background_jobs(8);

        let mut env = Env::new().unwrap();
        env.set_low_priority_background_threads(6);
        opts.set_env(&env);
        opts.set_max_open_files(1024);

        let mut opts2 = opts.clone();

        let mut block_opts = BlockBasedOptions::default();
        // blockdb caches = 2 GiB
        let cache = Cache::new_lru_cache(2 * GB);
        block_opts.set_block_cache(&cache);
        block_opts.set_block_size(16 * 1024);
        block_opts.set_cache_index_and_filter_blocks(true);
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        block_opts.set_ribbon_filter(9.9);
        opts.set_block_based_table_factory(&block_opts);

        let mut path = p.as_ref().to_owned();
        let mut path2 = path.clone();
        path.push("queue");
        path2.push("states");

        // 1 + 2 = 3 GiB roughly for this db
        let _ = DB::destroy(&opts, &path);
        let db = DB::open(&opts, &path)?;

        opts2.set_merge_operator_associative("min", min_merge);

        let mut block_opts2 = BlockBasedOptions::default();
        // blockdb caches = 5 GiB
        let blockdb_cache = Cache::new_lru_cache(5 * GB);
        block_opts2.set_block_cache(&blockdb_cache);
        block_opts2.set_block_size(16 * 1024);
        block_opts2.set_cache_index_and_filter_blocks(true);
        block_opts2.set_pin_l0_filter_and_index_blocks_in_cache(true);
        block_opts2.set_ribbon_filter(9.9);
        opts2.set_block_based_table_factory(&block_opts2);

        let cf_opts = opts2.clone();
        opts2.set_memtable_whole_key_filtering(true);
        opts2.create_missing_column_families(true);

        let bestcf = ColumnFamilyDescriptor::new(BEST, cf_opts.clone());
        let nextcf = ColumnFamilyDescriptor::new(NEXT, cf_opts);

        // Same 1 + 2 = 3 GiB for this one
        let _ = DB::destroy(&opts2, &path2);
        let statedb = DB::open_cf_descriptors(&opts2, &path2, vec![bestcf, nextcf])?;

        let mut write_opts = WriteOptions::default();
        write_opts.disable_wal(true);

        let s = Instant::now();
        let scorer = ContextScorer::shortest_paths(world, startctx, 32_768);
        log::info!("Built scorer in {:?}", s.elapsed());

        let max_possible_progress = W::NUM_CANON_LOCATIONS;
        let mut min_db_estimates = Vec::new();
        min_db_estimates.resize_with(max_possible_progress + 1, || u32::MAX.into());

        Ok(HeapDB {
            scorer,
            db,
            statedb,
            _cache: cache,
            _state_cache: blockdb_cache,
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
            next: 0.into(),
            iskips: 0.into(),
            pskips: 0.into(),
            dup_iskips: 0.into(),
            dup_pskips: 0.into(),
            deletes: 0.into(),
            delete: 0.into(),
            min_db_estimates,
            bg_deletes: 0.into(),
            retrieve_lock: Mutex::new(()),
            solutions,
            world,
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

    pub fn db_best(&self, progress: usize) -> u32 {
        self.min_db_estimates[progress].load(Ordering::Acquire)
    }

    pub fn db_bests(&self) -> Vec<u32> {
        self.min_db_estimates
            .iter()
            .map(|a| a.load(Ordering::Acquire))
            .collect()
    }

    pub fn min_progress(&self) -> Option<usize> {
        self.min_db_estimates
            .iter()
            .position(|a| a.load(Ordering::Acquire) != u32::MAX)
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

    fn next_cf(&self) -> &ColumnFamily {
        self.statedb.cf_handle(NEXT).unwrap()
    }

    /// The key for a ContextWrapper<T> in the queue is:
    /// the progress (4 bytes)
    /// the score (4 bytes),
    /// the elapsed time (4 bytes),
    /// a sequence number (4 bytes)
    fn get_heap_key_from_wrapper(&self, el: &ContextWrapper<T>) -> [u8; 16] {
        self.get_heap_key(el.get(), el.elapsed())
    }

    fn get_heap_key(&self, el: &T, elapsed: u32) -> [u8; 16] {
        let mut key: [u8; 16] = [0; 16];
        let progress: u32 = el.count_visits() as u32;
        let est = self.estimated_remaining_time(el);
        key[0..4].copy_from_slice(&progress.to_be_bytes());
        key[4..8].copy_from_slice(&(elapsed + est).to_be_bytes());
        key[8..12].copy_from_slice(&elapsed.to_be_bytes());
        key[12..16].copy_from_slice(&self.seq.fetch_add(1, Ordering::AcqRel).to_be_bytes());
        key
    }

    fn get_score_from_heap_key(key: &[u8]) -> u32 {
        u32::from_be_bytes(key[4..8].try_into().unwrap())
    }

    fn new_heap_key(&self, old_key: &[u8], old_elapsed: u32, new_elapsed: u32) -> [u8; 16] {
        let old_score = Self::get_score_from_heap_key(old_key);
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

    /// The key for a T (Ctx) in the statedb, and the value in the queue db
    /// are all T itself.
    pub(crate) fn serialize_state(el: &T) -> Vec<u8> {
        let mut key = Vec::with_capacity(std::mem::size_of::<T>());
        el.serialize(&mut Serializer::new(&mut key)).unwrap();
        key
    }

    fn deserialize_state(buf: &[u8]) -> Result<T, Error> {
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
        let ctx = Self::deserialize_state(value)?;
        let sd = self
            .get_deserialize_state_data(value)?
            .expect("Got unrecognized state from db!");
        Ok(ContextWrapper::with_elapsed(ctx, sd.elapsed))
    }

    fn get_deserialize_state_data(&self, key: &[u8]) -> Result<Option<StateDataAlias<T>>, Error> {
        match self.statedb.get_cf(self.best_cf(), key)? {
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

    fn serialize_next_data(next_entries: Vec<NextData>) -> Vec<u8> {
        Self::serialize_data(next_entries)
    }

    fn get_deserialize_next_data(&self, key: &[u8]) -> Result<Vec<NextData>, Error> {
        match self.statedb.get_cf(self.next_cf(), key)? {
            Some(slice) => Ok(Self::get_obj_from_data(&slice)?),
            None => Ok(Vec::new()),
        }
    }

    /// Estimates the remaining time to the goal.
    pub fn estimated_remaining_time(&self, ctx: &T) -> u32 {
        self.scorer.estimate_remaining_time(ctx).try_into().unwrap()
    }

    pub fn estimate_time_to_get(
        &self,
        ctx: &T,
        required: Vec<<<W as World>::Location as Location>::LocId>,
        subsets: Vec<(
            HashSet<<<W as World>::Location as Location>::LocId, CommonHasher>,
            i16,
        )>,
    ) -> u32 {
        self.scorer
            .estimate_time_to_get(ctx, required, subsets)
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
        if !self.record_one(&mut el, prev, false)? {
            return Ok(());
        }
        if el.elapsed() > max_time || self.score(&el) > max_time {
            self.iskips.fetch_add(1, Ordering::Release);
            return Ok(());
        }
        let key = self.get_heap_key_from_wrapper(&el);
        let val = Self::serialize_state(el.get());
        self.db.put_opt(key, val, &self.write_opts)?;
        self.size.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn push_from_queue(&self, el: ContextWrapper<T>, score: u32) -> Result<(), Error> {
        let progress = el.get().count_visits();
        let key = self.get_heap_key_from_wrapper(&el);
        let val = Self::serialize_state(el.get());
        self.db.put_opt(key, val, &self.write_opts)?;
        self.size.fetch_add(1, Ordering::Release);
        self.min_db_estimates[progress].fetch_min(score, Ordering::Release);
        Ok(())
    }

    pub fn pop(&self, start_progress: usize) -> anyhow::Result<Option<(T, u32)>> {
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

            let raw = u64::from_be_bytes(<[u8] as AsRef<[u8]>>::as_ref(&key[0..8]).try_into().unwrap()) + 1;
            // Ignore error
            let _ = self.db.delete_opt(&key, &self.write_opts);
            self.delete.fetch_max(raw, Ordering::Release);
            self.size.fetch_sub(1, Ordering::Release);

            if ndeletes % 20000 == 0 {
                let start = Instant::now();
                let max_deleted = self.delete.swap(0, Ordering::Acquire);
                self.db
                    .compact_range(None::<&[u8]>, Some(&max_deleted.to_be_bytes()));
                log::debug!("Compacting took {:?}", start.elapsed());
            }

            let el = Self::deserialize_state(&value)?;
            let elapsed = self.get_best_elapsed_raw(&value)?;
            if elapsed > self.max_time() {
                self.pskips.fetch_add(1, Ordering::Release);
                continue;
            }

            if self.remember_processed_raw(&value)? {
                self.dup_pskips.fetch_add(1, Ordering::Release);
                continue;
            }

            // Set the min score of this progress to this element
            // as an approximation
            let to_progress: usize = u32::from_be_bytes(key[0..4].try_into().unwrap())
                .try_into()
                .unwrap();
            // We use the key's cached version of score since our estimates
            // are based on the keys.
            let score = Self::get_score_from_heap_key(key.as_ref());

            self.reset_estimates_in_range(start_progress, to_progress, score);

            // We don't need to check the elapsed time against statedb,
            // because that's where the elapsed time came from
            return Ok(Some((el, elapsed)));
        }

        self.reset_estimates_in_range_unbounded(start_progress);

        Ok(None)
    }

    pub fn extend_from_queue<I>(&self, iter: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (T, u32)>,
    {
        let mut batch = WriteBatchWithTransaction::<false>::default();
        let max_time = self.max_time();
        let mut skips = 0;
        let mut dups = 0;

        let mut mins = Vec::new();
        mins.resize(W::NUM_CANON_LOCATIONS, u32::MAX);

        for (el, score) in iter {
            let elapsed = self.get_best_elapsed(&el)?;
            if elapsed > max_time || score > max_time {
                skips += 1;
                continue;
            }

            let val = Self::serialize_state(&el);

            if self.remember_processed_raw(&val).unwrap() {
                dups += 1;
                continue;
            }

            let progress = el.count_visits();
            mins[progress] = std::cmp::min(mins[progress], score);
            let key = self.get_heap_key(&el, elapsed);
            batch.put(key, val);
        }
        let new = batch.len();
        self.db.write_opt(batch, &self.write_opts)?;
        for (est, min) in self.min_db_estimates.iter().zip(mins.into_iter()) {
            est.fetch_min(min, Ordering::Release);
        }

        self.pskips.fetch_add(skips, Ordering::Release);
        self.dup_pskips.fetch_add(dups, Ordering::Release);
        self.size.fetch_add(new, Ordering::Release);

        Ok(())
    }

    /// Resets some min_db_estimates based on removed elements in a range.
    fn reset_estimates_in_range(&self, start_progress: usize, to_progress: usize, score: u32) {
        self.min_db_estimates[to_progress].store(score, Ordering::SeqCst);
        // If we went far enough that we got another progress level, the other ones have nothing left.
        for p in start_progress..to_progress {
            self.min_db_estimates[p].store(u32::MAX, Ordering::SeqCst);
        }
    }

    /// Resets some min_db_estimates based on never finding more elements.
    fn reset_estimates_in_range_unbounded(&self, start_progress: usize) {
        for p in start_progress..=W::NUM_CANON_LOCATIONS {
            self.min_db_estimates[p].store(u32::MAX, Ordering::SeqCst);
        }
    }

    /// Peeks in the db to reset min_db_estimates
    fn reset_estimates_actual(&self) {
        for p in 0..=W::NUM_CANON_LOCATIONS {
            let progress: u32 = p.try_into().unwrap();
            let mut tail_opts = ReadOptions::default();
            tail_opts.set_tailing(true);
            tail_opts.set_pin_data(true);
            tail_opts.set_iterate_lower_bound(progress.to_be_bytes());
            tail_opts.set_iterate_upper_bound((progress + 1).to_be_bytes());
            let mut iter = self.db.iterator_opt(IteratorMode::Start, tail_opts);
            if let Some(item) = iter.next() {
                let (key, _) = item.unwrap();
                let score = Self::get_score_from_heap_key(key.as_ref());
                self.min_db_estimates[p].store(score, Ordering::SeqCst);
            } else {
                self.min_db_estimates[p].store(u32::MAX, Ordering::SeqCst);
            }
        }
    }

    /// Retrieves up to `count` elements from the database, removing them.
    pub fn retrieve(
        &self,
        start_progress: usize,
        count: usize,
        score_limit: u32,
    ) -> Result<Vec<(T, u32, u32)>, Error> {
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
        let score = Self::get_score_from_heap_key(key.as_ref());
        batch.delete(key);

        let el = Self::deserialize_state(&value)?;
        let elapsed = self.get_best_elapsed_raw(&value)?;
        let est = self.estimated_remaining_time(&el);
        let max_time = self.max_time();
        if elapsed > max_time || elapsed + est > max_time {
            pskips += 1;
        } else if score > score_limit {
            res.push((el, elapsed, est));
            return Ok(res);
        } else {
            res.push((el, elapsed, est));
        }

        let start = Instant::now();
        'outer: while res.len() < count {
            loop {
                if let Some(item) = iter.next() {
                    let (key, value) = item.unwrap();
                    let score = Self::get_score_from_heap_key(key.as_ref());
                    batch.delete(key);
                    pops += 1;

                    let el = Self::deserialize_state(&value)?;
                    let elapsed = self.get_best_elapsed_raw(&value)?;
                    let est = self.estimated_remaining_time(&el);
                    let max_time = self.max_time();
                    if elapsed > max_time || elapsed + est > max_time {
                        pskips += 1;
                        continue;
                    }
                    if self.remember_processed_raw(&value)? {
                        dup_pskips += 1;
                        continue;
                    }

                    res.push((el, elapsed, est));
                    if res.len() == count {
                        break 'outer;
                    }
                    if score > score_limit {
                        break 'outer;
                    }
                } else {
                    break 'outer;
                }
            }
        }
        log::debug!(
            "We got {} results in {:?}, having iterated through {} elements",
            res.len(),
            start.elapsed(),
            pops
        );

        if let Some((el, elapsed, est)) = res.last() {
            self.reset_estimates_in_range(start_progress, el.count_visits(), elapsed + est);
        } else {
            self.reset_estimates_in_range_unbounded(start_progress);
        }

        // Ignore/assert errors once we start deleting.
        log::trace!("Beginning point deletion of iterated elements...");
        let start = Instant::now();
        self.db.write_opt(batch, &self.write_opts).unwrap();
        log::trace!("Deletes completed in {:?}", start.elapsed());

        self.size.fetch_sub(pops, Ordering::Release);
        self.pskips.fetch_add(pskips, Ordering::Release);
        self.dup_pskips.fetch_add(dup_pskips, Ordering::Release);

        Ok(res)
    }

    fn remember_processed_raw(&self, key: &[u8]) -> Result<bool, Error> {
        let cf = self.next_cf();
        Ok(
            self.statedb.key_may_exist_cf(cf, key)
                && self.statedb.get_pinned_cf(cf, key)?.is_some(),
        )
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
        self.get_best_elapsed_raw(&state_key)
    }

    fn get_best_elapsed_raw(&self, state_key: &[u8]) -> Result<u32, Error> {
        let sd = self
            .get_deserialize_state_data(state_key)?
            .expect("Didn't find state data!");
        Ok(sd.elapsed)
    }

    fn record_one_internal(
        &self,
        state_key: Vec<u8>,
        el: &mut ContextWrapper<T>,
        prev: &Vec<u8>,
        // In case the route looped back on itself, we use the best previous time
        best_elapsed: u32,
        next_entries: &mut Vec<NextData>,
    ) {
        let (hist, dur) = el.remove_history();
        next_entries.push((dur, state_key.clone()));

        assert!(
            hist.len() < TOO_MANY_STEPS,
            "Generated a state with way too much history: {}. Last 24:\n{:?}",
            hist.len(),
            hist.iter().skip(hist.len() - 24).collect::<Vec<_>>()
        );

        // This is the only part of the chain where the hist and prev are changed
        self.statedb
            .merge_cf_opt(
                self.best_cf(),
                &state_key,
                Self::serialize_data(StateData {
                    elapsed: best_elapsed,
                    hist,
                    prev: prev.clone(),
                }),
                &self.write_opts,
            )
            .unwrap();

        let mut to_adjust: Vec<_> = vec![(best_elapsed, state_key)];

        while let Some((prev_elapsed, state_key)) = to_adjust.pop() {
            // Get all the children of state_key
            // these are (dur, state) pairs that indicate states you get after el
            // and how long the transition to that state takes
            for (new_dur, new_ctx_key) in self.get_deserialize_next_data(&state_key).unwrap() {
                // new_dur = time after el
                let new_elapsed = prev_elapsed + new_dur;
                // each child points back at the prev and includes the hist step(s)
                if let Some(StateData {
                    elapsed,
                    hist,
                    prev,
                }) = self.get_deserialize_state_data(&new_ctx_key).unwrap()
                {
                    if new_elapsed < elapsed {
                        self.statedb
                            .merge_cf_opt(
                                self.best_cf(),
                                &new_ctx_key,
                                Self::serialize_data(StateData {
                                    elapsed: new_elapsed,
                                    // hist and prev don't change
                                    hist,
                                    prev,
                                }),
                                &self.write_opts,
                            )
                            .unwrap();
                        // handle solution by just inserting a new one
                        let ctx = Self::deserialize_state(&new_ctx_key).unwrap();
                        if self.world.won(&ctx) {
                            let sol = Arc::new(Solution {
                                elapsed: new_elapsed,
                                history: self.get_history_raw(new_ctx_key.clone()).unwrap(),
                            });
                            if self
                                .solutions
                                .lock()
                                .unwrap()
                                .insert_solution(sol)
                                .accepted()
                            {
                                log::info!(
                                    "New solution found by db improvement: {}ms",
                                    new_elapsed
                                );
                            }
                        }

                        // It doesn't really matter the order in which we update,
                        // but we shouldn't load everything all at once.
                        to_adjust.push((new_elapsed, new_ctx_key));
                    }
                }
                // We don't know what to write in the else case
                // since we haven't captured hist and prev...
            }
        }
    }

    /// Stores the underlying Ctx in the seen db with the best known elapsed time and
    /// its related history is also stored in the db,
    /// and returns whether this context had that best time.
    /// The Wrapper object is modified to reference the stored history.
    /// A `false` value means the state should be skipped.
    pub fn record_one(
        &self,
        el: &mut ContextWrapper<T>,
        prev: &Option<T>,
        state_only: bool,
    ) -> Result<bool, Error> {
        let state_key = Self::serialize_state(el.get());

        let (prev_key, best_elapsed) = if let Some(c) = prev {
            let prev_key = Self::serialize_state(c);
            let elapsed = self
                .get_deserialize_state_data(&prev_key)
                .unwrap()
                .map_or(el.elapsed(), |sd| sd.elapsed + el.recent_dur());
            (prev_key, elapsed)
        } else {
            (Vec::new(), el.elapsed())
        };

        let is_new =
            // TODO: Maybe we can make this deserialization cheaper as we only need one field?
            if let Some(StateData { elapsed, .. }) = self.get_deserialize_state_data(&state_key)? {
                // This is a new state being pushed, as it has new history, hence we skip if equal.
                if elapsed <= best_elapsed {
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
        self.record_one_internal(state_key, el, &prev_key, best_elapsed, &mut next_entries);

        if let Some(p) = prev {
            if !state_only {
                self.statedb
                    .put_cf_opt(
                        self.next_cf(),
                        Self::serialize_state(p),
                        Self::serialize_next_data(next_entries),
                        &self.write_opts,
                    )
                    .unwrap();
                self.next.fetch_add(1, Ordering::Release);
            }
        }

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
        let mut results = Vec::with_capacity(vec.len());
        let mut dups = 0;
        let mut new_seen = 0;
        let cf = self.best_cf();

        let (prev_key, prev_elapsed) = if let Some(c) = prev {
            let prev_key = Self::serialize_state(c);
            let elapsed = self
                .get_deserialize_state_data(&prev_key)
                .unwrap()
                .map(|sd| sd.elapsed);
            (prev_key, elapsed)
        } else {
            (Vec::new(), None)
        };

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
            let best_elapsed = if let Some(p_elapsed) = prev_elapsed {
                p_elapsed + el.recent_dur()
            } else {
                el.elapsed()
            };
            if let Some(StateData { elapsed, .. }) = seen_val {
                // This is a new state being pushed, as it has new history, hence we skip if equal.
                if elapsed <= best_elapsed {
                    results.push(false);
                    dups += 1;
                    continue;
                }
            } else {
                new_seen += 1;
            }
            // In every other case (no such state, or we do better than that state),
            // we will rewrite the data.
            self.record_one_internal(state_key, el, &prev_key, best_elapsed, &mut next_entries);
            results.push(true);
        }

        if let Some(p) = prev {
            self.statedb
                .put_cf_opt(
                    self.next_cf(),
                    Self::serialize_state(p),
                    Self::serialize_next_data(next_entries),
                    &self.write_opts,
                )
                .unwrap();
            self.next.fetch_add(1, Ordering::Release);
        }

        self.dup_iskips.fetch_add(dups, Ordering::Release);
        self.seen.fetch_add(new_seen, Ordering::Release);
        Ok(results)
    }

    pub fn cleanup(&self, batch_size: usize, exit_signal: &AtomicBool) -> Result<(), Error> {
        let mut start_key: Option<Box<[u8]>> = None;

        let mut end = false;
        let mut empty_passes = 0;
        while !end && !exit_signal.load(Ordering::Acquire) {
            let mut iter_opts = ReadOptions::default();
            iter_opts.set_tailing(true);
            iter_opts.fill_cache(false);
            let start_progress = if let Some(skey) = start_key {
                let p = u32::from_be_bytes(<[u8] as AsRef<[u8]>>::as_ref(&skey[0..4]).try_into().unwrap());
                iter_opts.set_iterate_lower_bound(skey);
                p
            } else {
                0
            };
            let mut iter = self.db.iterator_opt(IteratorMode::Start, iter_opts);

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

                    let known = u32::from_be_bytes(<[u8] as AsRef<[u8]>>::as_ref(&key[8..12]).try_into().unwrap());
                    if known > elapsed {
                        let new_key = self.new_heap_key(&key, known, elapsed);
                        batch.put(new_key, value);
                        batch.delete(&key);
                        rescores += 1;
                    }
                    if !compact && self._cache.get_usage() > 2 * GB {
                        compact = true;
                    }
                } else {
                    compact = true;
                    end = true;
                    break;
                }
            }
            start_key = iter.next().map(|p| p.unwrap().0);
            self.db.write_opt(batch, &self.write_opts).unwrap();
            self.reset_estimates_actual();
            drop(_retrieve_lock);
            if end && count == batch_size {
                log::debug!(
                    "Bg thread reached end at round start, left in db: {}",
                    self.size.load(Ordering::Acquire)
                );
                empty_passes += 1;
                assert!(
                    empty_passes < 10,
                    "Bg thread encountered too many empty passes in a row"
                );
                std::thread::sleep(std::time::Duration::from_secs(2));
            } else {
                empty_passes = 0;
            }
            self.pskips.fetch_add(pskips, Ordering::Release);
            self.dup_pskips.fetch_add(dup_pskips, Ordering::Release);
            self.bg_deletes
                .fetch_add(pskips + dup_pskips, Ordering::Release);
            self.size.fetch_sub(pskips + dup_pskips, Ordering::Release);
            if pskips > 0 || dup_pskips > 0 || rescores > 0 {
                log::debug!(
                    "Background thread (from prog={}): {} expired, {} duplicate, {} rescored",
                    start_progress,
                    pskips,
                    dup_pskips,
                    rescores
                );
            }
            if compact {
                let start = Instant::now();
                self.db.compact_range(None::<&[u8]>, None::<&[u8]>);
                log::debug!("Bg thread compacting took {:?}", start.elapsed());
            }
        }
        Ok(())
    }

    fn quick_detect_2cycle(&self, state_key: &Vec<u8>) -> Result<(), Error> {
        if let Some(StateData { hist, prev, .. }) = self.get_deserialize_state_data(state_key)? {
            if let Some(StateData {
                hist: hist2,
                prev: prev2,
                ..
            }) = self.get_deserialize_state_data(&prev)?
            {
                assert!(
                    prev2 != *state_key,
                    "2-cycle detected: last:{:?} prev:{:?}\nlast state: {:?}\nprev state: {:?}",
                    hist,
                    hist2,
                    state_key,
                    prev
                );
            }
        }
        Ok(())
    }

    fn detect_cycle(&self, mut state_key: Vec<u8>) -> Result<(), Error> {
        let mut states_found: HashMap<Vec<u8>, i32, CommonHasher> = new_hashmap();
        let mut depth = 0;
        loop {
            states_found.insert(state_key.clone(), depth);
            if let Some(StateData { hist, prev, .. }) =
                self.get_deserialize_state_data(&state_key)?
            {
                depth += 1;
                if let Some(existing_depth) = states_found.get(&prev) {
                    panic!(
                        "Cycle of length {} found ending at depth {}: last:{:?}\nstate: {:?}",
                        depth - existing_depth,
                        existing_depth,
                        hist,
                        Self::deserialize_state(&state_key)
                            .expect("Failed to deserialize while reporting an error")
                    );
                }
                state_key = prev;
            } else {
                return Ok(());
            }
        }
    }

    fn get_history_raw(&self, mut state_key: Vec<u8>) -> Result<Vec<HistoryAlias<T>>, Error> {
        assert!(self.quick_detect_2cycle(&state_key).is_ok());
        let mut vec = Vec::new();
        loop {
            if let Some(StateData { hist, prev, .. }) =
                self.get_deserialize_state_data(&state_key)?
            {
                if !prev.is_empty() {
                    assert!(
                        hist.len() < TOO_MANY_STEPS,
                        "History entry found in statedb way too long: {}. Last 24:\n{:?}",
                        hist.len(),
                        hist.iter().skip(hist.len() - 24).collect::<Vec<_>>()
                    );
                    if vec.len() >= TOO_MANY_STEPS {
                        assert!(self.detect_cycle(state_key).is_ok());
                    }
                    assert!(
                        vec.len() < TOO_MANY_STEPS,
                        "Raw history found in statedb way too long ({}), possible loop. Last 24:\n{:?}",
                        vec.len(),
                        vec.iter().skip(vec.len() - 24).collect::<Vec<_>>()
                    );
                    state_key = prev;
                    vec.push(hist);
                } else {
                    break;
                }
            } else {
                return Err(Error {
                    message: format!(
                        "Could not find state entry for {:?}",
                        Self::deserialize_state(&state_key)
                            .expect("Failed to deserialize while reporting an error")
                    ),
                });
            }
        }
        vec.reverse();
        Ok(vec.into_iter().flatten().collect())
    }

    pub fn get_history(&self, ctx: &T) -> Result<Vec<HistoryAlias<T>>, Error> {
        let state_key = Self::serialize_state(ctx);
        self.get_history_raw(state_key)
    }

    pub fn get_last_history_step(
        &self,
        ctx: &ContextWrapper<T>,
    ) -> Result<Option<HistoryAlias<T>>, Error> {
        if let Some(h) = ctx.recent_history().last() {
            Ok(Some(*h))
        } else {
            Ok(self
                .get_deserialize_state_data(&Self::serialize_state(ctx.get()))?
                .and_then(|sd| sd.hist.last().copied()))
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
        let dbstats = perf::get_memory_usage_stats(Some(&[&self.db]), Some(&[&self._cache]))?;
        let statestats =
            perf::get_memory_usage_stats(Some(&[&self.statedb]), Some(&[&self._state_cache]))?;

        Ok(format!(
            "db: total={}, unflushed={}, readers={}, caches={}, \
             cache={}, pinned={}\n\
             statedb: total={}, unflushed={}, readers={}, caches={}, \
             cache={}, pinned={}",
            SizeFormatter::new(dbstats.mem_table_total, BINARY),
            SizeFormatter::new(dbstats.mem_table_unflushed, BINARY),
            SizeFormatter::new(dbstats.mem_table_readers_total, BINARY),
            SizeFormatter::new(dbstats.cache_total, BINARY),
            SizeFormatter::new(self._cache.get_usage(), BINARY),
            SizeFormatter::new(self._cache.get_pinned_usage(), BINARY),
            SizeFormatter::new(statestats.mem_table_total, BINARY),
            SizeFormatter::new(statestats.mem_table_unflushed, BINARY),
            SizeFormatter::new(statestats.mem_table_readers_total, BINARY),
            SizeFormatter::new(statestats.cache_total, BINARY),
            SizeFormatter::new(self._state_cache.get_usage(), BINARY),
            SizeFormatter::new(self._state_cache.get_pinned_usage(), BINARY),
        ))
    }
}
