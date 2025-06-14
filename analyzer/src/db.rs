//! Wrapper around rocksdb with logic-graph specific features.
extern crate rocksdb;

use crate::context::*;
use crate::estimates::ContextScorer;
use crate::matchertrie::{MatcherRocksDb, MatcherTrieDb};
use crate::observer::short_observations;
use crate::route::{PartialRoute, RouteStep};
use crate::scoring::*;
use crate::steiner::*;
use crate::world::*;
use crate::{new_hashmap, CommonHasher};
use anyhow::{Error, Result};
use humansize::{SizeFormatter, BINARY};
use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins, Plot};
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use rmp_serde::{Deserializer, Serializer};
use rocksdb::{
    perf, BlockBasedOptions, Cache, ColumnFamily, ColumnFamilyDescriptor, Env, IteratorMode,
    MergeOperands, Options, ReadOptions, WriteBatchWithTransaction, WriteOptions, DB,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Range;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

const KB: usize = 1 << 10;
const MB: usize = 1 << 20;
const GB: usize = 1 << 30;
const BEST: &str = "best";
const NEXT: &str = "next";
const ROUTE: &str = "route";
const TRIE: &str = "trie";
const TOO_MANY_STEPS: usize = 1024 << 3;

// We need the following in this wrapper impl:
// 1. The queue db is mainly iterated over, via either
//    getting the minimum-score element (i.e. iterating from start)
//    or running over the whole db (e.g. for statistics). BlockDB is best for this.
// 2. We'll add an LRU cache layer that must outlive the BlockDB.

// We will have the following DBs:
// 1. the queue: (progress, elapsed, seq) -> Ctx
// 2. next: (Ctx, history step) -> (elapsed, Ctx)
// 3. best: Ctx -> (elapsed, history step, prev Ctx)

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

/// The key for a T (Ctx) in the statedb, and the value in the queue db
/// are all T itself.
pub(crate) fn serialize_state<T: Ctx>(el: &T) -> Vec<u8> {
    let mut key = Vec::with_capacity(std::mem::size_of::<T>());
    el.serialize(&mut Serializer::new(&mut key)).unwrap();
    key
}
fn deserialize_state<T: Ctx>(buf: &[u8]) -> Result<T> {
    Ok(rmp_serde::from_slice::<T>(buf)?)
}
pub fn serialize_data<V>(v: V) -> Vec<u8>
where
    V: Serialize,
{
    let mut val = Vec::with_capacity(std::mem::size_of::<V>());
    v.serialize(&mut Serializer::new(&mut val)).unwrap();
    val
}

fn get_obj_from_data<V>(buf: &[u8]) -> Result<V>
where
    V: for<'de> Deserialize<'de>,
{
    Ok(rmp_serde::from_slice::<V>(buf)?)
}

// Essentially a workaround for inherent associated types.
pub trait HeapMetric {
    type Score: Copy + Debug + Ord;
}

// This is a vec because we don't guarantee that the recent history in a newly submitted ctx
// is length 1.
type NextData<T> = Vec<HistoryAlias<T>>;
pub type NextSteps<T> = Vec<Vec<HistoryAlias<T>>>;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct StateData<I, S, L, E, A, Wp> {
    // Ordering is important here, since min_merge will sort by serialized bytes.
    elapsed: u32,
    time_since_visit: u32,
    estimated_remaining: u32,
    hist: Vec<History<I, S, L, E, A, Wp>>,
    // This is mainly going to be used for performing another lookup, no point
    // in deserializing just to reserialize
    prev: Vec<u8>,
}

impl<I, S, L, E, A, Wp> StateData<I, S, L, E, A, Wp> {
    pub fn best_times(&self) -> BestTimes {
        BestTimes {
            elapsed: self.elapsed,
            time_since_visit: self.time_since_visit,
            estimated_remaining: self.estimated_remaining,
        }
    }
}

type StateDataAlias<T> = StateData<
    <T as Ctx>::ItemId,
    <<<T as Ctx>::World as World>::Exit as Exit>::SpotId,
    <<<T as Ctx>::World as World>::Location as Location>::LocId,
    <<<T as Ctx>::World as World>::Exit as Exit>::ExitId,
    <<<T as Ctx>::World as World>::Action as Action>::ActionId,
    <<<T as Ctx>::World as World>::Warp as Warp>::WarpId,
>;

pub struct HeapDB<'w, W: World + 'w, T: Ctx, const KS: usize, SM> {
    db: DB,
    statedb: DB,
    _cache: Cache,
    _state_cache: Cache,
    write_opts: WriteOptions,

    max_time: AtomicU32,

    metric: SM,
    recovery: AtomicBool,
    size: AtomicUsize,
    seen: AtomicUsize,
    next: AtomicUsize,
    iskips: AtomicUsize,
    pskips: AtomicUsize,
    dup_iskips: AtomicUsize,
    dup_pskips: AtomicUsize,
    readds: AtomicUsize,

    deletes: AtomicUsize,
    delete: AtomicU64,

    min_db_estimates: Vec<AtomicU32>,

    bg_deletes: AtomicUsize,

    retrieve_lock: Mutex<()>,
    phantom: PhantomData<&'w (W, T)>,
}

impl<'w, W, T, L, E, const KS: usize, SM> HeapMetric for HeapDB<'w, W, T, KS, SM>
where
    W: World<Location = L, Exit = E> + 'w,
    T: Ctx<World = W>,
    L: Location<Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = E::Currency>,
    SM: ScoreMetric<'w, W, T, KS>,
{
    type Score = SM::Score;
}

impl<'w, W, T, L, E, const KS: usize, SM> HeapDB<'w, W, T, KS, SM>
where
    W: World<Location = L, Exit = E> + 'w,
    T: Ctx<World = W>,
    L: Location<Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = E::Currency>,
    SM: ScoreMetric<'w, W, T, KS>,
{
    pub fn open<P>(
        p: P,
        initial_max_time: u32,
        metric: SM,
        delete_first: bool,
    ) -> Result<HeapDB<'w, W, T, KS, SM>>
    where
        P: AsRef<Path>,
    {
        let mut opts = Options::default();
        opts.create_if_missing(true);
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

        let recovery = if delete_first {
            let _ = DB::destroy(&opts, &path);
            let _ = DB::destroy(&opts2, &path2);
            false
        } else if std::fs::exists(&path2)? {
            log::debug!("Restoring some queue elements from existing db");
            true
        } else {
            false
        };

        // 1 + 2 = 3 GiB roughly for this db
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
        let statedb = DB::open_cf_descriptors(&opts2, &path2, vec![bestcf, nextcf])?;

        let mut write_opts = WriteOptions::default();
        write_opts.disable_wal(true);

        let max_possible_progress = W::NUM_CANON_LOCATIONS;
        let mut min_db_estimates = Vec::new();
        min_db_estimates.resize_with(max_possible_progress + 1, || u32::MAX.into());

        let seen = statedb
            .property_int_value("estimate-num-keys")?
            .unwrap_or(0) as usize;

        Ok(HeapDB {
            db,
            statedb,
            _cache: cache,
            _state_cache: blockdb_cache,
            write_opts,
            max_time: initial_max_time.into(),
            metric,
            recovery: recovery.into(),
            size: 0.into(),
            seen: seen.into(),
            next: 0.into(),
            iskips: 0.into(),
            pskips: 0.into(),
            dup_iskips: 0.into(),
            dup_pskips: 0.into(),
            readds: 0.into(),
            deletes: 0.into(),
            delete: 0.into(),
            min_db_estimates,
            bg_deletes: 0.into(),
            retrieve_lock: Mutex::new(()),
            phantom: PhantomData,
        })
    }

    pub fn scorer(
        &self,
    ) -> &ContextScorer<
        W,
        <<W as World>::Exit as Exit>::SpotId,
        <<W as World>::Location as Location>::LocId,
        <<W as World>::Location as Location>::CanonId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    > {
        self.metric.estimator()
    }

    /// Returns the number of elements in the heap (tracked separately from the db).
    pub fn len(&self) -> usize {
        self.size.load(Ordering::Acquire)
    }

    pub fn is_empty(&self) -> bool {
        self.size.load(Ordering::Acquire) == 0
    }

    pub fn recovery(&self) -> bool {
        self.recovery.load(Ordering::Acquire)
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
        self.metric.estimates()
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
        self.metric.cached_estimates()
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

    /// Returns the number of states that were over the max time and then improved below it and
    /// so re-added to the queue.
    pub fn readds(&self) -> usize {
        self.readds.load(Ordering::Acquire)
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

    pub fn lookup_score(&self, el: &T) -> Result<SM::Score> {
        Ok(self.metric.score_from_times(self.get_best_times(el)?))
    }

    pub fn lookup_score_raw(&self, key: &[u8]) -> Result<SM::Score> {
        Ok(self.metric.score_from_times(self.get_best_times_raw(key)?))
    }

    pub fn metric(&self) -> &SM {
        &self.metric
    }

    /// The key for a ContextWrapper<T> in the queue is:
    /// the progress (4 bytes)
    /// the score (4 bytes),
    /// the total time estimate (4 bytes),
    /// a sequence number (4 bytes)
    fn get_heap_key_from_wrapper_score(
        &self,
        el: &ContextWrapper<T>,
        score: SM::Score,
    ) -> [u8; KS] {
        self.metric.get_heap_key(el.get(), score)
    }

    fn get_queue_entry_wrapper(&self, value: &[u8]) -> Result<ContextWrapper<T>> {
        let ctx = deserialize_state(value)?;
        let sd = self
            .get_deserialize_state_data(value)?
            .expect("Got unrecognized state from db!");
        Ok(ContextWrapper::with_times(
            ctx,
            sd.elapsed,
            sd.time_since_visit,
        ))
    }

    fn get_deserialize_state_data(&self, key: &[u8]) -> Result<Option<StateDataAlias<T>>> {
        match self.statedb.get_cf(self.best_cf(), key)? {
            Some(slice) => Ok(Some(get_obj_from_data(&slice)?)),
            None => Ok(None),
        }
    }

    fn get_state_values<'a, I>(
        &self,
        cf: &ColumnFamily,
        state_keys: I,
    ) -> Result<Vec<Option<StateDataAlias<T>>>>
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
                Ok(Some(slice)) => Ok(Some(get_obj_from_data(&slice).unwrap())),
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
            Err(Error::msg(error.join("; ")))
        } else {
            Ok(parsed.into_iter().map(|res| res.unwrap()).collect())
        }
    }

    fn serialize_next_data(next_entries: Vec<NextData<T>>) -> Vec<u8> {
        serialize_data(next_entries)
    }

    fn get_deserialize_next_data(&self, key: &[u8]) -> Result<Vec<NextData<T>>> {
        match self.statedb.get_cf(self.next_cf(), key)? {
            Some(slice) => Ok(get_obj_from_data(&slice)?),
            None => Ok(Vec::new()),
        }
    }

    /// Estimates the remaining time to the goal.
    pub fn estimated_remaining_time(&self, ctx: &T) -> u32 {
        self.metric.estimated_remaining_time(ctx)
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
        self.metric
            .estimator()
            .estimate_time_to_get(ctx, required, subsets)
            .try_into()
            .unwrap()
    }

    /// Pushes an element into the db.
    /// If the element's elapsed time is greater than the allowed maximum,
    /// or, if the state has been previously processed or previously seen
    /// with an equal or lower elapsed time, does nothing.
    pub fn push(&self, mut el: ContextWrapper<T>, prev: &Option<T>) -> Result<()> {
        let max_time = self.max_time();
        // Records the history in the statedb, even if over time.
        let Some(score) = self.record_one(&mut el, prev, false)? else {
            return Ok(());
        };
        if el.elapsed() > max_time || self.metric.total_estimate_from_score(score) > max_time {
            self.iskips.fetch_add(1, Ordering::Release);
            return Ok(());
        }
        let key = self.get_heap_key_from_wrapper_score(&el, score);
        let val = serialize_state(el.get());
        self.db.put_opt(key, val, &self.write_opts)?;
        self.size.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub fn push_from_queue(&self, el: ContextWrapper<T>, score: SM::Score) -> Result<()> {
        let progress = el.get().count_visits();
        let key = self.get_heap_key_from_wrapper_score(&el, score);
        let val = serialize_state(el.get());
        self.db.put_opt(key, val, &self.write_opts)?;
        self.size.fetch_add(1, Ordering::Release);
        let primary = self.metric.score_primary(score);
        self.min_db_estimates[progress].fetch_min(primary, Ordering::Release);
        Ok(())
    }

    pub fn pop(&self, start_progress: usize) -> anyhow::Result<Option<(T, u32, u32)>> {
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

            let raw = u64::from_be_bytes(
                <[u8] as AsRef<[u8]>>::as_ref(&key[0..8])
                    .try_into()
                    .unwrap(),
            ) + 1;
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

            let el = deserialize_state(&value)?;
            let BestTimes {
                elapsed,
                time_since_visit,
                ..
            } = self.get_best_times_raw(&value)?;
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
            let score = SM::get_score_primary_from_heap_key(key.as_ref());

            self.reset_estimates_in_range(start_progress, to_progress, score);

            // We don't need to check the elapsed time against statedb,
            // because that's where the elapsed time came from
            return Ok(Some((el, elapsed, time_since_visit)));
        }

        self.reset_estimates_in_range_unbounded(start_progress);

        Ok(None)
    }

    pub fn extend_from_queue<I>(&self, iter: I) -> Result<()>
    where
        I: IntoIterator<Item = (T, SM::Score)>,
    {
        let mut batch = WriteBatchWithTransaction::<false>::default();
        let max_time = self.max_time();
        let mut skips = 0;
        let mut dups = 0;

        let mut mins = Vec::new();
        mins.resize(W::NUM_CANON_LOCATIONS, u32::MAX);

        for (el, _) in iter {
            let score = self.lookup_score(&el)?;
            if self.metric.total_estimate_from_score(score) > max_time {
                skips += 1;
                continue;
            }

            let val = serialize_state(&el);

            if self.remember_processed_raw(&val).unwrap() {
                dups += 1;
                continue;
            }

            let progress = el.count_visits();
            mins[progress] = std::cmp::min(mins[progress], self.metric.score_primary(score));
            let key = self.metric.get_heap_key(&el, score);
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
                let score = SM::get_score_primary_from_heap_key(key.as_ref());
                self.min_db_estimates[p].store(score, Ordering::SeqCst);
            } else {
                self.min_db_estimates[p].store(u32::MAX, Ordering::SeqCst);
            }
        }
    }

    /// Retrieves up to `count` elements from the database, removing them.
    /// Elements are returned as a tuple (T, elapsed time, score, estimated remaining time)
    pub fn retrieve(
        &self,
        start_progress: usize,
        count: usize,
        score_limit: u32,
    ) -> Result<Vec<(T, SM::Score)>> {
        let _retrieve_lock = self.retrieve_lock.lock().unwrap();
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
        let pscore = SM::get_score_primary_from_heap_key(key.as_ref());
        batch.delete(key);

        let mut res = Vec::with_capacity(count);
        let el = deserialize_state(&value)?;
        let score = self.lookup_score_raw(&value)?;
        let max_time = self.max_time();
        if self.metric.total_estimate_from_score(score) > max_time {
            pskips += 1;
        // TODO: Not sure if we need a score limit when score is time_since?
        } else if pscore > score_limit {
            res.push((el, score));
            log::debug!(
                "Returning immediately with one element (pscore {} > limit {})",
                pscore,
                score_limit
            );
            return Ok(res);
        } else {
            res.push((el, score));
        }

        let start = Instant::now();
        'outer: while res.len() < count {
            loop {
                if let Some(item) = iter.next() {
                    let (key, value) = item.unwrap();
                    let pscore = SM::get_score_primary_from_heap_key(key.as_ref());
                    batch.delete(key);
                    pops += 1;

                    let el = match deserialize_state(&value) {
                        Ok(el) => el,
                        Err(e) => {
                            log::error!("Corrupt value in queue: {}\n{:?}", e, value);
                            continue;
                        }
                    };
                    let score = self.lookup_score_raw(&value)?;
                    let max_time = self.max_time();
                    if self.metric.total_estimate_from_score(score) > max_time {
                        pskips += 1;
                        continue;
                    }
                    if self.remember_processed_raw(&value)? {
                        dup_pskips += 1;
                        continue;
                    }

                    res.push((el, score));
                    if res.len() == count {
                        break 'outer;
                    }
                    if pscore > score_limit {
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

        if let Some((el, score)) = res.last() {
            self.reset_estimates_in_range(
                start_progress,
                el.count_visits(),
                self.metric.score_primary(*score),
            );
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

    fn remember_processed_raw(&self, key: &[u8]) -> Result<bool> {
        let cf = self.next_cf();
        Ok(
            self.statedb.key_may_exist_cf(cf, key)
                && self.statedb.get_pinned_cf(cf, key)?.is_some(),
        )
    }

    /// Checks whether the given Ctx was already processed into its next states.
    pub fn remember_processed(&self, el: &T) -> Result<bool> {
        let next_key = serialize_state(el);
        self.remember_processed_raw(&next_key)
    }

    fn get_next_steps_raw(&self, key: &[u8]) -> Result<NextSteps<T>> {
        let cf = self.next_cf();
        Ok(if self.statedb.key_may_exist_cf(cf, key) {
            self.get_deserialize_next_data(key)?
        } else {
            Vec::new()
        })
    }

    pub fn get_next_steps(&self, el: &T) -> Result<NextSteps<T>> {
        self.get_next_steps_raw(&serialize_state(el))
    }

    pub fn count_duplicate(&self) {
        self.dup_pskips.fetch_add(1, Ordering::Release);
    }

    pub fn get_best_times(&self, el: &T) -> Result<BestTimes> {
        let state_key = serialize_state(el);
        self.get_best_times_raw(&state_key)
    }

    fn get_best_times_raw(&self, state_key: &[u8]) -> Result<BestTimes> {
        let sd = self
            .get_deserialize_state_data(state_key)?
            .expect("Didn't find state data!");
        Ok(sd.best_times())
    }

    fn record_one_internal(
        &self,
        state_key: &Vec<u8>,
        el: &mut ContextWrapper<T>,
        prev: &Vec<u8>,
        // In case the route looped back on itself, we use the best previous times
        best_since_visit: u32,
        best_elapsed: u32,
        estimated_remaining: u32,
        next_entries: &mut Vec<NextData<T>>,
    ) {
        let (hist, _) = el.remove_history();
        next_entries.push(hist.clone());

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
                state_key,
                serialize_data(StateData {
                    elapsed: best_elapsed,
                    time_since_visit: best_since_visit,
                    estimated_remaining,
                    hist,
                    prev: prev.clone(),
                }),
                &self.write_opts,
            )
            .unwrap();
    }

    /// Stores the underlying Ctx in the seen db with the best known elapsed time and
    /// its related history is also stored in the db,
    /// and returns the score of the state, or None if it was not the best time (and should be skipped).
    /// The Wrapper object is modified to reference the stored history.
    pub fn record_one(
        &self,
        el: &mut ContextWrapper<T>,
        prev: &Option<T>,
        state_only: bool,
    ) -> Result<Option<SM::Score>> {
        let state_key = serialize_state(el.get());

        let (prev_key, best_since_from_prev, best_elapsed_from_prev) = if let Some(c) = prev {
            let prev_key = serialize_state(c);
            if let Some(sd) = self.get_deserialize_state_data(&prev_key).unwrap() {
                (
                    prev_key,
                    // If recent_dur is larger than time_since_visit, then it means
                    // that we had a visit in the recent history.
                    // Otherwise, we didn't, so we can just add the recent_dur.
                    if el.time_since_visit() < el.recent_dur() {
                        el.time_since_visit()
                    } else {
                        sd.time_since_visit + el.recent_dur()
                    },
                    sd.elapsed + el.recent_dur(),
                )
            } else {
                (prev_key, el.time_since_visit(), el.elapsed())
            }
        } else {
            (Vec::new(), el.time_since_visit(), el.elapsed())
        };

        let (is_new, old_elapsed, estimated_remaining) = if let Some(StateData {
            elapsed,
            estimated_remaining,
            ..
        }) =
            self.get_deserialize_state_data(&state_key)?
        {
            // This is a new state being pushed, as it has new history, hence we skip if equal.
            if elapsed <= best_elapsed_from_prev {
                self.dup_iskips.fetch_add(1, Ordering::Release);
                return Ok(None);
            }
            (false, elapsed, estimated_remaining)
        } else {
            // state not seen before, determine time remaining
            (true, 0, self.metric.estimated_remaining_time(el.get()))
        };
        // In every other case (no such state, or we do better than that state),
        // we will rewrite the data.

        // We should also check the StateData for whether we even need to do this
        let mut next_entries = Vec::new();
        self.record_one_internal(
            &state_key,
            el,
            &prev_key,
            best_since_from_prev,
            best_elapsed_from_prev,
            estimated_remaining,
            &mut next_entries,
        );

        if let Some(p) = prev {
            if !state_only {
                self.statedb
                    .put_cf_opt(
                        self.next_cf(),
                        serialize_state(p),
                        Self::serialize_next_data(next_entries),
                        &self.write_opts,
                    )
                    .unwrap();
                self.next.fetch_add(1, Ordering::Release);
            }
        }

        let score = self.metric.score_from_times(BestTimes {
            elapsed: best_elapsed_from_prev,
            time_since_visit: best_since_from_prev,
            estimated_remaining,
        });

        if is_new {
            self.seen.fetch_add(1, Ordering::Release);
        } else {
            let max_time = self.max_time();
            // If it was an improvement just over the max_time, and it hasn't been processed yet,
            // add it to the queue
            if old_elapsed >= max_time
                && best_elapsed_from_prev < max_time
                && !self.remember_processed_raw(&state_key)?
            {
                let qkey = self.metric.get_heap_key(el.get(), score);
                self.db.put_opt(qkey, state_key, &self.write_opts)?;
                self.readds.fetch_add(1, Ordering::Release);
                self.size.fetch_add(1, Ordering::Release);
            }
        }
        Ok(Some(score))
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
    ) -> Result<Vec<Option<SM::Score>>> {
        let max_time = self.max_time();
        let mut next_entries = Vec::new();
        let mut results = Vec::with_capacity(vec.len());
        let mut dups = 0;
        let mut new_seen = 0;
        let cf = self.best_cf();

        vec.sort_by_key(ContextWrapper::elapsed);
        let (prev_key, prev_scoreinfo) = if let Some(c) = prev {
            let prev_key = serialize_state(c);
            let scoreinfo = self
                .get_deserialize_state_data(&prev_key)
                .unwrap()
                .map(|sd| (sd.time_since_visit, sd.elapsed));
            (prev_key, scoreinfo)
        } else {
            (Vec::new(), None)
        };

        let seeing: Vec<_> = vec.iter().map(|el| serialize_state(el.get())).collect();

        let seen_values = self.get_state_values(cf, seeing.iter())?;

        for ((el, state_key), seen_val) in vec
            .iter_mut()
            .zip(seeing.into_iter())
            .zip(seen_values.into_iter())
        {
            let (best_since_visit, best_elapsed) =
                if let Some((p_since_visit, p_elapsed)) = prev_scoreinfo {
                    (
                        // If recent_dur is larger than time_since_visit, then it means
                        // that we had a visit in the recent history.
                        // Otherwise, we didn't, so we can just add the recent_dur.
                        if el.time_since_visit() < el.recent_dur() {
                            el.time_since_visit()
                        } else {
                            p_since_visit + el.recent_dur()
                        },
                        p_elapsed + el.recent_dur(),
                    )
                } else {
                    (el.time_since_visit(), el.elapsed())
                };
            let (old_elapsed, estimated_remaining) = if let Some(StateData {
                elapsed,
                estimated_remaining,
                ..
            }) = seen_val
            {
                // This is a new state being pushed, as it has new history, hence we skip if equal.
                if elapsed <= best_elapsed {
                    results.push(None);
                    dups += 1;
                    continue;
                }
                (elapsed, estimated_remaining)
            } else {
                // state not seen before, determine time remaining
                new_seen += 1;
                (0, self.metric.estimated_remaining_time(el.get()))
            };
            // In every other case (no such state, or we do better than that state),
            // we will rewrite the data.
            self.record_one_internal(
                &state_key,
                el,
                &prev_key,
                best_since_visit,
                best_elapsed,
                estimated_remaining,
                &mut next_entries,
            );

            let score = self.metric.score_from_times(BestTimes {
                elapsed: best_elapsed,
                time_since_visit: best_since_visit,
                estimated_remaining,
            });
            results.push(Some(score));

            // If it was an improvement just over the max_time, and it hasn't been processed yet,
            // add it to the queue
            if old_elapsed >= max_time
                && best_elapsed < max_time
                && !self.remember_processed_raw(&state_key)?
            {
                let qkey = self.metric.get_heap_key(el.get(), score);
                self.db.put_opt(qkey, state_key, &self.write_opts)?;
                self.readds.fetch_add(1, Ordering::Release);
                self.size.fetch_add(1, Ordering::Release);
            }
        }

        if let Some(p) = prev {
            self.statedb
                .put_cf_opt(
                    self.next_cf(),
                    serialize_state(p),
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

    pub fn cleanup(&self, batch_size: usize, exit_signal: &AtomicBool) -> Result<()> {
        let mut start_key: Option<Box<[u8]>> = None;

        let mut end = false;
        let mut empty_passes = 0;
        while !end && !exit_signal.load(Ordering::Acquire) {
            let mut iter_opts = ReadOptions::default();
            iter_opts.set_tailing(true);
            iter_opts.fill_cache(false);
            let start_progress = if let Some(skey) = start_key {
                let p = u32::from_be_bytes(
                    <[u8] as AsRef<[u8]>>::as_ref(&skey[0..4])
                        .try_into()
                        .unwrap(),
                );
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

                    let new_score = self
                        .lookup_score_raw(&value)
                        .expect("Error reading state in bg thread");
                    let max_time = self.max_time();
                    if self.metric.total_estimate_from_score(new_score) > max_time {
                        batch.delete(key);
                        pskips += 1;
                        continue;
                    }

                    let old_score = self.metric.score_from_heap_key(&key);
                    if old_score > new_score {
                        let new_key = self.metric.new_heap_key(&key, new_score);
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

    pub fn restore(&self) {
        if !self.recovery.load(Ordering::Acquire) {
            return;
        }

        let state_snapshot = self.statedb.snapshot();

        self.reset_estimates_actual();
        let mut iter_opts = ReadOptions::default();
        iter_opts.fill_cache(false);
        let iter = state_snapshot.iterator_cf_opt(self.best_cf(), iter_opts, IteratorMode::Start);
        let next_cf = self.next_cf();
        for state_el in iter {
            let (key, val) = state_el.unwrap();
            if state_snapshot
                .get_pinned_cf(next_cf, key.as_ref())
                .is_ok_and(|o| o.is_some())
            {
                self.next.fetch_add(1, Ordering::Release);
            } else {
                let state: T = get_obj_from_data(key.as_ref()).unwrap();
                let data: StateDataAlias<T> = get_obj_from_data(val.as_ref()).unwrap();
                let score = self.metric().score_from_times(data.best_times());
                let heap_key_min = self.metric().get_heap_key(&state, score);
                if self
                    .db
                    .put_opt(&heap_key_min, serialize_state(&state), &self.write_opts)
                    .is_ok()
                {
                    self.size.fetch_add(1, Ordering::Release);
                }
            }
        }

        self.recovery.store(false, Ordering::Release);
        log::info!("Finished scanning state table for restore");
    }

    fn quick_detect_2cycle(&self, state_key: &Vec<u8>) -> Result<()> {
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

    fn detect_cycle(&self, mut state_key: Vec<u8>) -> Result<()> {
        let mut states_found: HashMap<Vec<u8>, usize, CommonHasher> = new_hashmap();
        let mut depth = 0;
        let mut hist_vec = Vec::new();
        loop {
            states_found.insert(state_key.clone(), depth);
            if let Some(StateData { hist, prev, .. }) =
                self.get_deserialize_state_data(&state_key)?
            {
                hist_vec.push(hist);
                depth += 1;
                if let Some(existing_depth) = states_found.get(&prev) {
                    let hist = &hist_vec[*existing_depth..depth];
                    panic!(
                        "Cycle of length {} found ending at depth {}:\n{:?}\nstate: {:?}",
                        depth - existing_depth,
                        existing_depth,
                        hist.into_iter().rev().collect::<Vec<_>>(),
                        deserialize_state::<T>(&state_key)
                            .expect("Failed to deserialize while reporting an error")
                    );
                }
                state_key = prev;
            } else {
                return Ok(());
            }
        }
    }

    fn get_history_raw(&self, mut state_key: Vec<u8>) -> Result<(Vec<HistoryAlias<T>>, u32)> {
        assert!(self.quick_detect_2cycle(&state_key).is_ok());
        let mut vec = Vec::new();
        let Some(StateData {
            elapsed,
            mut hist,
            mut prev,
            ..
        }) = self.get_deserialize_state_data(&state_key)?
        else {
            return Err(Error::msg(format!(
                "Could not find state entry for {:?}",
                deserialize_state::<T>(&state_key)
                    .expect("Failed to deserialize while reporting an error")
            )));
        };
        while !prev.is_empty() {
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
            if let Some(next) = self.get_deserialize_state_data(&state_key)? {
                hist = next.hist;
                prev = next.prev;
            } else {
                return Err(Error::msg(format!(
                    "Could not find intermediate state entry for {:?}",
                    deserialize_state::<T>(&state_key)
                        .expect("Failed to deserialize while reporting an error")
                )));
            }
        }

        vec.reverse();
        Ok((vec.into_iter().flatten().collect(), elapsed))
    }

    pub fn get_history(&self, ctx: &T) -> Result<(Vec<HistoryAlias<T>>, u32)> {
        let state_key = serialize_state(ctx);
        self.get_history_raw(state_key)
    }

    pub fn get_last_history_step(
        &self,
        ctx: &ContextWrapper<T>,
    ) -> Result<Option<HistoryAlias<T>>> {
        if let Some(h) = ctx.recent_history().last() {
            Ok(Some(*h))
        } else {
            Ok(self
                .get_deserialize_state_data(&serialize_state(ctx.get()))?
                .and_then(|sd| sd.hist.last().copied()))
        }
    }

    pub fn print_graphs(&self) -> Result<()> {
        let size = self.size.load(Ordering::Acquire);
        let max_time = self.max_time.load(Ordering::Acquire);
        let mut times: Vec<f64> = Vec::with_capacity(size);
        let mut time_scores: Vec<(f64, f64)> = Vec::with_capacity(size);
        let mut read_opts = ReadOptions::default();
        read_opts.fill_cache(false);
        let iter = self.db.iterator_opt(IteratorMode::Start, read_opts);
        for item in iter {
            let (key, value) = item?;
            let el = self.get_queue_entry_wrapper(&value)?;
            let score = self.metric().score_from_heap_key(&key);
            times.push(el.elapsed().into());
            time_scores.push((
                el.elapsed().into(),
                self.metric.total_estimate_from_score(score).into(),
            ));
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

    pub fn get_memory_usage_stats(&self) -> Result<String> {
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

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
struct SavedRoute {
    pub time: u32,
    pub route_id: usize,
    pub route_start: usize,
    pub route_end: usize,
}

impl PartialOrd for SavedRoute {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Ord for SavedRoute {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

pub struct RouteDb<T>
where
    T: Ctx,
    T::PropertyObservation: Serialize + for<'a> Deserialize<'a>,
{
    db: MatcherRocksDb<T, SavedRoute>,
    _cache: Cache,
    phantom: PhantomData<T>,

    next_route_id: AtomicUsize,
}

impl<T> RouteDb<T>
where
    T: Ctx,
    T::PropertyObservation: Serialize + for<'a> Deserialize<'a>,
{
    pub fn default_options() -> (Options, Cache) {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        // change compression options?
        // 4 write buffers at 256 MiB = 1 GiB
        opts.set_write_buffer_size(256 * MB);
        opts.set_max_write_buffer_number(4);
        opts.set_target_file_size_base(128 * MB as u64);
        // use half the logical cores, clamp between 2 and 32
        opts.increase_parallelism(std::cmp::max(
            2,
            std::cmp::min(num_cpus::get() / 2, 32).try_into().unwrap(),
        ));
        opts.set_max_background_jobs(4);

        let mut env = Env::new().unwrap();
        env.set_low_priority_background_threads(3);
        opts.set_env(&env);
        opts.set_max_open_files(512);

        let mut block_opts = BlockBasedOptions::default();
        // blockdb caches = 2 GiB
        let cache = Cache::new_lru_cache(2 * GB);
        block_opts.set_block_cache(&cache);
        block_opts.set_block_size(16 * KB);
        block_opts.set_cache_index_and_filter_blocks(true);
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        block_opts.set_ribbon_filter(9.9);
        opts.set_block_based_table_factory(&block_opts);
        (opts, cache)
    }

    pub fn test_options() -> (Options, Cache) {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_write_buffer_size(4 * KB);
        opts.set_max_write_buffer_number(4);

        let mut block_opts = BlockBasedOptions::default();
        let cache = Cache::new_lru_cache(16 * KB);
        block_opts.set_block_cache(&cache);
        block_opts.set_block_size(1 * KB);
        block_opts.set_cache_index_and_filter_blocks(true);
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        block_opts.set_ribbon_filter(9.9);
        opts.set_block_based_table_factory(&block_opts);
        (opts, cache)
    }

    pub fn open<P>(
        p: P,
        mut opts: Options,
        cache: Cache,
        delete_first: bool,
    ) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let cf_opts = opts.clone();
        let mut cf_opts_trie = opts.clone();
        cf_opts_trie.set_merge_operator_associative("min", min_merge);

        opts.set_memtable_whole_key_filtering(true);
        opts.create_missing_column_families(true);

        let mut path = p.as_ref().to_owned();
        path.push("routes");
        if delete_first {
            let _ = DB::destroy(&opts, &path);
        }

        let routecf = ColumnFamilyDescriptor::new(ROUTE, cf_opts);
        let triecf = ColumnFamilyDescriptor::new(TRIE, cf_opts_trie);
        let db = DB::open_cf_descriptors(&opts, &path, vec![routecf, triecf])?;

        let next_route_id = if !delete_first {
            // Read last key of route table to get next id
            let mut iter = db.raw_iterator_cf(db.cf_handle(ROUTE).unwrap());
            iter.seek_to_last();
            let key = iter.key();
            if let Some(id_raw) = key {
                // ROUTE cf keys are pairs (route id: usize, index: usize)
                get_obj_from_data::<(usize, usize)>(id_raw).unwrap().0 + 1
            } else {
                1
            }
        } else {
            1
        };

        Ok(RouteDb {
            db: MatcherRocksDb::from_db_cf(db, TRIE),
            _cache: cache,
            phantom: PhantomData::default(),
            next_route_id: next_route_id.into(),
        })
    }

    pub fn route_cf(&self) -> &ColumnFamily {
        self.db.db().cf_handle(ROUTE).unwrap()
    }

    pub fn trie_cf(&self) -> &ColumnFamily {
        self.db.db().cf_handle(TRIE).unwrap()
    }

    // for testing
    pub fn internal_db(&self) -> &DB {
        self.db.db()
    }

    pub fn route_key(key: &[u8]) -> (usize, usize) {
        get_obj_from_data::<(usize, usize)>(key).unwrap()
    }

    pub fn trie_key(key: &[u8]) -> String {
        let mut de = Deserializer::from_read_ref(key);
        let spot: <<T::World as World>::Exit as Exit>::SpotId =
            Deserialize::deserialize(&mut de).unwrap();
        let mut vec = Vec::<T::PropertyObservation>::new();
        while let Ok(obs) = Deserialize::deserialize(&mut de) {
            vec.push(obs);
        }
        format!("{:?}/{:?}", spot, vec)
    }

    pub fn num_routes(&self) -> usize {
        self.next_route_id.load(Ordering::Acquire)
    }

    pub fn trie_size(&self) -> usize {
        self.db.size()
    }

    pub fn insert_route<W>(
        &self,
        startctx: &T,
        world: &W,
        dest: <W::Exit as Exit>::SpotId,
        route: &PartialRoute<T>,
    ) -> usize
    where
        W: World,
        T: Ctx<World = W>,
        W::Location: Location<Context = T>,
    {
        let route_id = self.next_route_id.fetch_add(1, Ordering::Acquire);
        let prefix = serialize_data(dest);

        // 1. Batch-write keys like "{route_id}:{idx}" for each individual step
        // 2. For each observation set along the route, record "{dest}:{oset}" -> SavedRoute

        let mut batch = WriteBatchWithTransaction::<false>::default();
        let route_cf = self.route_cf();
        for (i, step) in route.iter().enumerate() {
            batch.put_cf(
                route_cf,
                serialize_data((route_id, i)),
                serialize_data(step),
            );
        }
        self.db.db().write(batch).unwrap();

        let observations = short_observations(
            startctx,
            world,
            &route.iter().map(|rs| rs.step).collect::<Vec<_>>(),
            false,
            false,
        );
        let mut batch = WriteBatchWithTransaction::<false>::default();
        let end = route.end;
        let mut time = 0;
        for (i, (rs, obs)) in route.iter().zip(observations).enumerate() {
            let saved = SavedRoute {
                route_id,
                route_start: i,
                route_end: end,
                time: route.time - time,
            };
            time += rs.time;
            self.db.insert_batch(&mut batch, obs, saved, &prefix);
        }
        // Should we put every subroute in? just the ones from the start or to the end? or only the end?
        self.db.db().write(batch).unwrap();

        route_id
    }

    pub fn best_known_route(
        &self,
        startctx: &T,
        dest: <<<T as Ctx>::World as World>::Exit as Exit>::SpotId,
    ) -> Result<Option<Vec<RouteStep<T>>>> {
        let prefix = serialize_data(dest);
        let routes = self.db.lookup(startctx, &prefix);
        if let Some(sr) = routes.iter().min_by_key(|sr| sr.time) {
            let route_cf = self.route_cf();
            let steps = self.db.db().multi_get_cf(
                Range {
                    start: sr.route_start,
                    end: sr.route_end,
                }
                .map(|idx| (route_cf, serialize_data((sr.route_id, idx)))),
            );
            let mut vec = Vec::with_capacity(sr.route_end - sr.route_start);
            for step in steps {
                vec.push(get_obj_from_data::<RouteStep<T>>(&step?.unwrap())?);
            }
            Ok(Some(vec))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod test {
    use super::StateData;
    use crate::context::History;

    type GenericStateData = StateData<u8, u8, u8, u8, u8, u8>;

    #[test]
    fn test_merge() {
        let sd1 = GenericStateData {
            elapsed: 123456,
            time_since_visit: 789,
            estimated_remaining: 6543,
            hist: Vec::new(),
            prev: Vec::new(),
        };

        let sd2 = GenericStateData {
            elapsed: 1111111,
            time_since_visit: 111,
            estimated_remaining: 1111,
            hist: vec![History::A(3)],
            prev: Vec::new(),
        };

        let sd3 = GenericStateData {
            elapsed: 1111111,
            time_since_visit: 7,
            estimated_remaining: 1211,
            hist: vec![History::A(3)],
            prev: Vec::new(),
        };

        let sd1_mp = super::serialize_data(sd1);
        let sd2_mp = super::serialize_data(sd2);
        let sd3_mp = super::serialize_data(sd3);

        println!("{:?}\n{:?}\n{:?}", sd1_mp, sd2_mp, sd3_mp);

        assert!(
            sd1_mp < sd2_mp,
            "Serialized data with less elapsed_time is greater lexicographically"
        );

        assert!(
            sd3_mp < sd2_mp,
            "Serialized data with less time_since is greater lexicographically"
        );
    }
}
