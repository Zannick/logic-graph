extern crate plotlib;

use crate::bucket::*;
use crate::context::*;
use crate::db::{HeapDB, HeapMetric};
use crate::estimates::ContextScorer;
use crate::scoring::{BestTimes, EstimatedTimeMetric, ScoreMetric, TimeSinceAndElapsed};
use crate::steiner::*;
use crate::world::*;
use anyhow::{anyhow, Result};
use bucket_queue::{Bucket, BucketQueue, Queue};
use log;
use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins, Plot};
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use sort_by_derive::SortBy;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

#[derive(Debug, SortBy)]
pub(crate) struct HeapElement<T: Ctx> {
    #[sort_by]
    pub(crate) score: u32,
    pub(crate) el: ContextWrapper<T>,
}

#[allow(unused)]
pub(self) type TimeSinceDbType<'w, W, T> = HeapDB<'w, W, T, 16, TimeSinceAndElapsed<'w, W>>;
#[allow(unused)]
pub(self) type ElapsedTimeDb<'w, W, T> = HeapDB<'w, W, T, 12, EstimatedTimeMetric<'w, W>>;

// These types have to be changed to affect the score type.
pub(crate) type MetricType<'w, W> = TimeSinceAndElapsed<'w, W>;
pub(self) type DbType<'w, W, T> = TimeSinceDbType<'w, W, T>;
// Automatic from DbType
pub(self) type Score<'w, W, T> = <DbType<'w, W, T> as HeapMetric>::Score;
pub struct RocksBackedQueue<'w, W, T>
where
    T: Ctx<World = W>,
    W: World,
    W::Location: Location<Context = T>,
{
    // TODO: Make the bucket element just T.
    queue: Mutex<BucketQueue<Segment<T, Score<'w, W, T>>>>,
    db: DbType<'w, W, T>,
    capacity: usize,
    iskips: AtomicUsize,
    pskips: AtomicUsize,
    min_evictions: usize,
    max_evictions: usize,
    min_reshuffle: usize,
    max_reshuffle: usize,
    evictions: AtomicUsize,
    retrievals: AtomicUsize,
    retrieving: AtomicBool,
    // Locked by queue!
    processed_counts: Vec<AtomicUsize>,
    world: &'w W,
}

impl<'w, W, T> RocksBackedQueue<'w, W, T>
where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
{
    pub fn new<P>(
        db_path: P,
        world: &'w W,
        metric: MetricType<'w, W>,
        initial_max_time: u32,
        max_capacity: usize,
        min_evictions: usize,
        max_evictions: usize,
        min_reshuffle: usize,
        max_reshuffle: usize,
        delete_dbs: bool,
    ) -> Result<RocksBackedQueue<'w, W, T>>
    where
        P: AsRef<Path>,
    {
        let db = HeapDB::open(db_path, initial_max_time, metric, delete_dbs)?;
        let max_possible_progress = W::NUM_CANON_LOCATIONS;
        let mut processed_counts = Vec::new();
        processed_counts.resize_with(max_possible_progress + 1, || 0.into());
        let q = RocksBackedQueue {
            queue: Mutex::new(BucketQueue::new()),
            db,
            capacity: max_capacity,
            iskips: 0.into(),
            pskips: 0.into(),
            min_evictions,
            max_evictions,
            min_reshuffle,
            max_reshuffle,
            evictions: 0.into(),
            retrievals: 0.into(),
            retrieving: false.into(),
            processed_counts,
            world,
        };
        Ok(q)
    }

    pub fn db(&self) -> &DbType<'w, W, T> {
        &self.db
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
        self.db.scorer()
    }

    pub fn heap_len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len() + self.db.len()
    }

    pub fn db_len(&self) -> usize {
        self.db.len()
    }

    pub fn seen(&self) -> usize {
        self.db.seen()
    }

    pub fn estimates(&self) -> usize {
        self.db.estimates()
    }

    pub fn cached_estimates(&self) -> usize {
        self.db.cached_estimates()
    }

    pub fn db_bests(&self) -> Vec<u32> {
        self.db.db_bests()
    }

    pub fn heap_bests(&self) -> Vec<Option<Score<'w, W, T>>> {
        let queue = self.queue.lock().unwrap();
        queue.peek_all_buckets_min()
    }

    pub fn min_progress(&self) -> Option<usize> {
        let queue = self.queue.lock().unwrap();
        queue.min_priority()
    }

    pub fn estimated_remaining_time(&self, ctx: &ContextWrapper<T>) -> u32 {
        self.db.estimated_remaining_time(ctx.get())
    }

    /// Returns whether the underlying queue and db are actually empty.
    /// Even if this returns false, attempting to peek or pop may produce None.
    pub fn is_empty(&self) -> bool {
        self.db.len() == 0 && self.queue.lock().unwrap().is_empty()
    }

    pub fn max_time(&self) -> u32 {
        self.db.max_time()
    }

    pub fn set_max_time(&self, max_time: u32) {
        self.db.set_max_time(max_time);
    }

    pub fn set_lenient_max_time(&self, max_time: u32) {
        self.db.set_lenient_max_time(max_time);
    }

    pub fn evictions(&self) -> usize {
        self.evictions.load(Ordering::Acquire)
    }

    pub fn retrievals(&self) -> usize {
        self.retrievals.load(Ordering::Acquire)
    }

    pub fn background_deletes(&self) -> usize {
        self.db.background_deletes()
    }

    /// Pushes an element into the queue.
    /// If the element's elapsed time is greater than the allowed maximum,
    /// or, the state has been previously seen with an equal or lower elapsed time, does nothing.
    pub fn push(&self, mut el: ContextWrapper<T>, prev: &Option<T>) -> Result<()> {
        let start = Instant::now();
        // Always record the state even if it is over time.
        let Some(score) = self.db.record_one(&mut el, prev, false)? else {
            return Ok(());
        };
        // Don't bother pushing winning states.
        if self.world.won(el.get()) {
            return Ok(());
        }
        if el.elapsed() > self.db.max_time() {
            self.iskips.fetch_add(1, Ordering::Release);
            return Ok(());
        }

        let total_estimate = self.db.metric().total_estimate_from_score(score);
        if total_estimate > self.db.max_time() {
            self.iskips.fetch_add(1, Ordering::Release);
            return Ok(());
        };
        let progress = el.get().count_visits();
        let mut evicted = None;
        {
            let mut queue = self.queue.lock().unwrap();

            if queue.len() == self.capacity {
                // compare to the last element, aka the MAX
                let (ctx, &p_max) = queue
                    .peek_segment_max(progress)
                    .ok_or(anyhow!("queue at capacity with no elements"))?;
                let BestTimes { elapsed, .. } = self.db.get_best_times(ctx)?;
                if score > p_max || (score == p_max && el.elapsed() >= elapsed) {
                    // Lower priority (or equal but later), evict the new item immediately
                    self.db.push_from_queue(el, score)?;
                } else {
                    let evictions = std::cmp::min(
                        self.max_evictions,
                        std::cmp::max(self.min_evictions, queue.len() / 2),
                    );
                    // New item is better, evict some old_items.
                    evicted = Some(Self::evict_internal(&mut queue, evictions));
                    queue.push(el.into_inner(), progress, score);
                }
            } else {
                queue.push(el.into_inner(), progress, score);
            }
        }
        // Without the lock (but still blocking the push op in this thread)
        if let Some(ev) = evicted {
            log::trace!("push+evict took {:?} with the lock", start.elapsed());
            if !ev.is_empty() {
                self.evict_to_db(ev, "push")?;
            }
        }

        Ok(())
    }

    fn evict_to_db(&self, ev: Vec<(T, Score<'w, W, T>)>, category: &str) -> Result<()> {
        let start = Instant::now();
        self.db.extend_from_queue(ev)?;
        self.evictions.fetch_add(1, Ordering::Release);
        log::debug!("{}:evict to db took {:?}", category, start.elapsed());
        log::debug!("{}", self.db.get_memory_usage_stats().unwrap());
        Ok(())
    }

    /// Removes elements from the max end of each segment in the queue until we reach
    /// the minimum desired evictions.
    fn evict_internal(
        queue: &mut MutexGuard<BucketQueue<Segment<T, Score<'w, W, T>>>>,
        min_evictions: usize,
    ) -> Vec<(T, Score<'w, W, T>)> {
        let evicted = queue.pop_max_proportionally(min_evictions);
        queue.shrink_to_fit();
        evicted
    }

    /// Retrieves up to the given number of elements for the given segment from the db.
    fn retrieve(
        &self,
        segment: usize,
        num: usize,
        score_limit: u32,
    ) -> Result<Vec<(T, usize, Score<'w, W, T>)>> {
        log::debug!(
            "Beginning retrieve of {} entries from segment {} and up, we have {} total in the db",
            num,
            segment,
            self.db.len()
        );
        let res: Vec<_> = self
            .db
            .retrieve(segment, num, score_limit)?
            .into_iter()
            .map(|(el, score)| {
                let progress = el.count_visits();
                (el, progress, score)
            })
            .collect();

        Ok(res)
    }

    pub fn pop(&self) -> Result<Option<ContextWrapper<T>>> {
        let mut queue = self.queue.lock().unwrap();
        while !queue.is_empty() || !self.db.is_empty() {
            while let Some((el, &min_score)) = queue.peek_min() {
                let progress = el.count_visits();
                let db_best = self.db.db_best(progress);
                // Only when we go a decent bit over
                if !self.db.is_empty()
                    && db_best < u32::MAX
                    && self.db.metric().score_primary(min_score) > db_best * 11 / 10
                {
                    queue = self.maybe_reshuffle(
                        progress,
                        self.min_reshuffle,
                        self.max_reshuffle,
                        queue,
                    )?;
                }
                let (ctx, _) = queue.pop_min().unwrap();

                // Retrieve the best elapsed time.
                let BestTimes {
                    elapsed,
                    time_since_visit,
                    estimated_remaining,
                } = self.db.get_best_times(&ctx)?;

                let max_time = self.db.max_time();
                if elapsed > max_time || elapsed + estimated_remaining > max_time {
                    self.pskips.fetch_add(1, Ordering::Release);
                    continue;
                }
                if self.db.remember_processed(&ctx)? {
                    self.db.count_duplicate();
                    continue;
                }
                self.processed_counts[progress].fetch_add(1, Ordering::Release);
                return Ok(Some(ContextWrapper::with_times(
                    ctx,
                    elapsed,
                    time_since_visit,
                )));
            }
            // Retrieve some from db
            if !self.db.is_empty() {
                if !self.retrieving.fetch_or(true, Ordering::AcqRel) {
                    queue = self.do_retrieve_and_insert(0, queue)?;
                    self.retrieving.store(false, Ordering::Release);
                } else {
                    return Ok(self.db.pop(0)?.map(|(el, elapsed, time_since_visit)| {
                        let progress = el.count_visits();
                        self.processed_counts[progress].fetch_add(1, Ordering::Release);
                        ContextWrapper::with_times(el, elapsed, time_since_visit)
                    }));
                }
            }
        }
        Ok(None)
    }

    fn maybe_reshuffle<'a>(
        &'a self,
        progress: usize,
        min_to_restore: usize,
        max_to_restore: usize,
        queue: MutexGuard<'a, BucketQueue<Segment<T, Score<'w, W, T>>>>,
    ) -> Result<MutexGuard<'a, BucketQueue<Segment<T, Score<'w, W, T>>>>> {
        if !self.retrieving.fetch_or(true, Ordering::AcqRel) {
            let r = self.maybe_reshuffle_locked(progress, min_to_restore, max_to_restore, queue);
            self.retrieving.store(false, Ordering::Release);
            r
        } else {
            Ok(queue)
        }
    }

    fn maybe_reshuffle_locked<'a>(
        &'a self,
        progress: usize,
        min_to_restore: usize,
        max_to_restore: usize,
        mut queue: MutexGuard<'a, BucketQueue<Segment<T, Score<'w, W, T>>>>,
    ) -> Result<MutexGuard<'a, BucketQueue<Segment<T, Score<'w, W, T>>>>> {
        let start = Instant::now();
        let num_buckets = queue.approx_num_buckets();
        // Get a decent amount to refill
        let num_to_restore = std::cmp::max(
            min_to_restore,
            std::cmp::min(
                max_to_restore,
                (self.capacity - queue.len()) / (num_buckets + 1),
            ),
        );
        let len = queue.len();
        let score_limit = if let Some((lower, upper)) = queue.peek_segment_priority_range(progress)
        {
            (self.db.metric().score_primary(*lower) + self.db.metric().score_primary(*upper)) / 2
        } else {
            self.max_time()
        };
        if score_limit < self.db.db_best(progress) {
            return Ok(queue);
        }
        if self.capacity - len < num_to_restore {
            // evict at least twice that much.
            let evicted = Self::evict_internal(
                &mut queue,
                std::cmp::max(self.min_evictions, 2 * num_to_restore),
            );
            log::trace!("reshuffle:evict took {:?}", start.elapsed());
            drop(queue);
            self.evict_to_db(evicted, "reshuffle")?;
        } else {
            drop(queue);
        }
        let res = self.retrieve(progress, num_to_restore, score_limit)?;
        self.retrievals.fetch_add(1, Ordering::Release);
        queue = self.queue.lock().unwrap();
        if !res.is_empty() {
            queue.extend(res);
            log::debug!("Reshuffle took total {:?}", start.elapsed());
            assert!(!queue.is_empty(), "Queue should have data after retrieve");
        }
        Ok(queue)
    }

    fn maybe_fetch_for_empty_buckets<'a>(
        &'a self,
        mut queue: MutexGuard<'a, BucketQueue<Segment<T, Score<'w, W, T>>>>,
    ) -> Result<MutexGuard<'a, BucketQueue<Segment<T, Score<'w, W, T>>>>> {
        // Runs over all the buckets
        let Some(min_score) = queue
            .peek_min()
            .map(|p| self.db.metric().score_primary(*p.1))
        else {
            return Ok(queue);
        };
        // threshold is 1/8 the difference between min score and a reasonable upper bound,
        // plus a small buffer for when min gets really close
        // the reasonable upper bound is max_time / max number of visits
        let bound = self.max_time() / W::NUM_CANON_LOCATIONS as u32;
        // if bound is LESS THAN min_score, then we very much should retrieve with even a small difference
        let threshold = (bound.saturating_sub(min_score)) / 8 + (min_score / 1024);

        // We retrieve if the difference between the db best and the min score is more than the threshold.
        // An alternate way to look at this is min_score - threshold = ms * 9/8 - ms / 1024 - upper bound
        // or db_best < min_score * 1151 / 1024 - upper bound
        // Since this could underflow, we add a short-circuit
        let db_threshold = if threshold < min_score {
            min_score - threshold
        } else {
            return Ok(queue);
        };
        for (progress, db_best) in self.db_bests().iter().enumerate() {
            // skip if bucket is not empty
            if queue
                .bucket_for_peeking(progress)
                .map(|bucket| !bucket.is_empty_bucket())
                .unwrap_or(false)
            {
                continue;
            }
            // skip if there's nothing in the db
            if *db_best == u32::MAX {
                continue;
            }

            if *db_best < db_threshold {
                queue = self.maybe_reshuffle(
                    progress,
                    self.min_reshuffle / 2,
                    self.max_reshuffle / 4,
                    queue,
                )?;
            }
        }
        Ok(queue)
    }

    fn do_retrieve_and_insert<'a>(
        &'a self,
        segment: usize,
        mut queue: MutexGuard<'a, BucketQueue<Segment<T, Score<'w, W, T>>>>,
    ) -> Result<MutexGuard<'a, BucketQueue<Segment<T, Score<'w, W, T>>>>> {
        let start = Instant::now();
        let num_to_restore = std::cmp::max(
            self.min_reshuffle,
            std::cmp::min(
                self.max_reshuffle,
                (self.capacity - queue.len()) / W::NUM_CANON_LOCATIONS,
            ),
        );
        drop(queue);
        let res = self.retrieve(segment, num_to_restore, self.max_time())?;
        queue = self.queue.lock().unwrap();
        queue.extend(res);
        self.retrievals.fetch_add(1, Ordering::Release);
        log::debug!("Retrieval took total {:?}", start.elapsed());
        Ok(queue)
    }

    fn pop_special<F>(&self, n: usize, pop_func: F) -> Result<Vec<ContextWrapper<T>>>
    where
        F: Fn(
            &mut MutexGuard<BucketQueue<Segment<T, Score<'w, W, T>>>>,
        ) -> Option<(T, Score<'w, W, T>)>,
    {
        let mut vec = Vec::new();
        let mut queue = self.queue.lock().unwrap();
        while vec.len() < n && (!queue.is_empty() || !self.db.is_empty()) {
            while let Some((ctx, _)) = (pop_func)(&mut queue) {
                // Retrieve the best elapsed time.
                let BestTimes {
                    elapsed,
                    time_since_visit,
                    estimated_remaining,
                } = self.db.get_best_times(&ctx)?;
                let max_time = self.db.max_time();
                if elapsed > max_time || elapsed + estimated_remaining > max_time {
                    self.pskips.fetch_add(1, Ordering::Release);
                    continue;
                }
                if self.db.remember_processed(&ctx)? {
                    self.db.count_duplicate();
                    continue;
                }
                let progress = ctx.count_visits();
                self.processed_counts[progress].fetch_add(1, Ordering::Release);

                vec.push(ContextWrapper::with_times(ctx, elapsed, time_since_visit));
                if vec.len() == n {
                    return Ok(vec);
                }
            }
            // Retrieve some from db
            if !self.db.is_empty() {
                if !self.retrieving.fetch_or(true, Ordering::AcqRel) {
                    queue = self.do_retrieve_and_insert(0, queue)?;
                    self.retrieving.store(false, Ordering::Release);
                } else if let Some((ctx, elapsed, time_since_visit)) = self.db.pop(0)? {
                    let progress = ctx.count_visits();
                    self.processed_counts[progress].fetch_add(1, Ordering::Release);
                    vec.push(ContextWrapper::with_times(ctx, elapsed, time_since_visit));
                } else {
                    return Ok(vec);
                }
            }
        }
        Ok(vec)
    }

    fn pop_special_multi<F>(&self, n: usize, pop_func: F) -> Result<Vec<ContextWrapper<T>>>
    where
        F: Fn(
            usize,
            &mut MutexGuard<BucketQueue<Segment<T, Score<'w, W, T>>>>,
        ) -> Vec<(T, Score<'w, W, T>)>,
    {
        let mut vec = Vec::new();
        let mut queue = self.queue.lock().unwrap();
        while vec.len() < n && (!queue.is_empty() || !self.db.is_empty()) {
            loop {
                let values = (pop_func)(n - vec.len(), &mut queue);
                if values.is_empty() {
                    break;
                }
                for (ctx, _) in values.into_iter() {
                    // Retrieve the best elapsed time.
                    let BestTimes {
                        elapsed,
                        time_since_visit,
                        estimated_remaining,
                    } = self.db.get_best_times(&ctx)?;
                    let max_time = self.db.max_time();
                    if elapsed > max_time || elapsed + estimated_remaining > max_time {
                        self.pskips.fetch_add(1, Ordering::Release);
                        continue;
                    }
                    if self.db.remember_processed(&ctx)? {
                        self.db.count_duplicate();
                        continue;
                    }
                    let progress = ctx.count_visits();
                    self.processed_counts[progress].fetch_add(1, Ordering::Release);

                    vec.push(ContextWrapper::with_times(ctx, elapsed, time_since_visit));
                }
                if vec.len() >= n {
                    return Ok(vec);
                }
            }
            // Retrieve some from db
            if !self.db.is_empty() {
                if !self.retrieving.fetch_or(true, Ordering::AcqRel) {
                    queue = self.do_retrieve_and_insert(0, queue)?;
                    self.retrieving.store(false, Ordering::Release);
                } else if let Some((ctx, elapsed, time_since_visit)) = self.db.pop(0)? {
                    let progress = ctx.count_visits();
                    self.processed_counts[progress].fetch_add(1, Ordering::Release);
                    vec.push(ContextWrapper::with_times(ctx, elapsed, time_since_visit));
                } else {
                    return Ok(vec);
                }
            }
        }
        Ok(vec)
    }

    pub fn pop_max_estimate(&self, n: usize) -> Result<Vec<ContextWrapper<T>>> {
        self.pop_special(n, |q| q.pop_max())
    }

    pub fn pop_min_progress(&self, progress: usize, n: usize) -> Result<Vec<ContextWrapper<T>>> {
        self.pop_special(n, |q| {
            let segment = progress + q.min_priority()?;
            q.pop_segment_min(segment)
                .or_else(|| q.pop_max_segment_min())
        })
    }

    pub fn pop_max_progress(&self, n: usize) -> Result<Vec<ContextWrapper<T>>> {
        self.pop_special(n, |q| q.pop_max_segment_min())
    }

    pub fn pop_half_progress(&self, n: usize) -> Result<Vec<ContextWrapper<T>>> {
        self.pop_special(n, |q| {
            let half = (q.min_priority()? + q.max_priority()?) / 2;
            q.pop_segment_min(half).or_else(|| q.pop_max_segment_min())
        })
    }

    pub fn pop_mode(&self, n: usize) -> Result<Vec<ContextWrapper<T>>> {
        self.pop_special_multi(n, |k, q| q.pop_n_from_largest_segment(k))
    }

    pub fn pop_local_minima(&self) -> Result<Vec<ContextWrapper<T>>> {
        let mut vec = Vec::new();
        let mut queue = self.queue.lock().unwrap();
        if queue.is_empty() {
            return Ok(vec);
        }

        let min = queue.min_priority().unwrap();
        let max = queue.max_priority().unwrap();
        let mut prev = queue
            .bucket_for_peeking(min)
            .and_then(|b| b.min_priority().copied());
        'next: for segment in (min + 1)..=max {
            let next = queue
                .bucket_for_peeking(segment)
                .and_then(|b| b.min_priority().copied());
            if let (Some(lower), Some(higher)) = (prev, next) {
                if higher < lower {
                    while let Some((ctx, _)) = queue.pop_segment_min(segment) {
                        // Retrieve the best elapsed time.
                        let BestTimes {
                            elapsed,
                            time_since_visit,
                            estimated_remaining,
                        } = self.db.get_best_times(&ctx)?;
                        let max_time = self.db.max_time();
                        if elapsed > max_time || elapsed + estimated_remaining > max_time {
                            self.pskips.fetch_add(1, Ordering::Release);
                            continue;
                        }
                        if self.db.remember_processed(&ctx)? {
                            self.db.count_duplicate();
                            continue;
                        }
                        let progress = ctx.count_visits();
                        self.processed_counts[progress].fetch_add(1, Ordering::Release);
                        vec.push(ContextWrapper::with_times(ctx, elapsed, time_since_visit));
                        continue 'next;
                    }
                }
            }
            prev = next;
        }

        Ok(vec)
    }

    pub fn pop_round_robin(&self, min_priority: usize) -> Result<Vec<ContextWrapper<T>>> {
        let mut queue = self.queue.lock().unwrap();
        let mut did_retrieve = false;
        queue = self.maybe_fetch_for_empty_buckets(queue)?;
        while !queue.is_empty() || !self.db.is_empty() {
            if let Some(min) = queue.min_priority() {
                let min = std::cmp::max(min, min_priority);
                let max = queue.max_priority().unwrap();
                if max < min {
                    break;
                }
                let mut diffs = Vec::with_capacity(max - min + 1);

                let mut vec = Vec::with_capacity(max - min + 1);
                'next: for segment in min..=max {
                    if let Some(b) = queue.bucket_for_peeking(segment) {
                        if b.is_empty_bucket() {
                            continue;
                        }
                        let db_best = self.db.db_best(segment);
                        while let Some((ctx, _)) =
                            queue.bucket_for_removing(segment).unwrap().pop_min()
                        {
                            // Retrieve the best elapsed time.
                            let BestTimes {
                                elapsed,
                                time_since_visit,
                                estimated_remaining,
                            } = self.db.get_best_times(&ctx)?;
                            let max_time = self.db.max_time();

                            // We won't actually use what is added here right away, unless we drop the element we just popped.
                            if !did_retrieve
                                && !self.db.is_empty()
                                && db_best < u32::MAX
                                && time_since_visit
                                    > db_best
                                        + max_time
                                            / std::cmp::max(128, W::NUM_CANON_LOCATIONS as u32)
                            {
                                diffs.push((segment, time_since_visit - db_best));
                            }

                            if elapsed > max_time || elapsed + estimated_remaining > max_time {
                                self.pskips.fetch_add(1, Ordering::Release);
                                continue;
                            }
                            if self.db.remember_processed(&ctx)? {
                                self.db.count_duplicate();
                                continue;
                            }

                            let progress = ctx.count_visits();
                            self.processed_counts[progress].fetch_add(1, Ordering::Release);
                            vec.push(ContextWrapper::with_times(ctx, elapsed, time_since_visit));
                            continue 'next;
                        }
                        if db_best < u32::MAX {
                            if self.retrieving.fetch_or(true, Ordering::AcqRel) {
                                continue 'next;
                            }
                            queue = self.do_retrieve_and_insert(segment, queue)?;
                            self.retrieving.store(false, Ordering::Release);
                            did_retrieve = true;

                            // Just grab the next one
                            while let Some((ctx, _)) = queue.pop_segment_min(segment) {
                                // Retrieve the best elapsed time.
                                let BestTimes {
                                    elapsed,
                                    time_since_visit,
                                    estimated_remaining,
                                } = self.db.get_best_times(&ctx)?;
                                let max_time = self.db.max_time();
                                if elapsed > max_time || elapsed + estimated_remaining > max_time {
                                    self.pskips.fetch_add(1, Ordering::Release);
                                    continue;
                                }
                                if self.db.remember_processed(&ctx)? {
                                    self.db.count_duplicate();
                                    continue;
                                }
                                let progress = ctx.count_visits();
                                self.processed_counts[progress].fetch_add(1, Ordering::Release);
                                vec.push(ContextWrapper::with_times(
                                    ctx,
                                    elapsed,
                                    time_since_visit,
                                ));
                                continue 'next;
                            }
                        }
                    }
                }
                // Max one retrieve per pop_round_robin
                if !did_retrieve {
                    if let Some((segment, _)) = diffs.into_iter().max_by_key(|p| p.1) {
                        let _queue = self.maybe_reshuffle(
                            segment,
                            self.min_reshuffle,
                            self.max_reshuffle,
                            queue,
                        )?;
                    }
                }
                vec.shrink_to_fit();
                return Ok(vec);
            } else {
                // Retrieve some from db
                if !self.db.is_empty() {
                    if !self.retrieving.fetch_or(true, Ordering::AcqRel) {
                        queue = self.do_retrieve_and_insert(0, queue)?;
                        self.retrieving.store(false, Ordering::Release);
                    } else if let Some((el, elapsed, time_since_visit)) = self.db.pop(0)? {
                        let progress = el.count_visits();
                        self.processed_counts[progress].fetch_add(1, Ordering::Release);
                        return Ok(vec![ContextWrapper::with_times(
                            el,
                            elapsed,
                            time_since_visit,
                        )]);
                    }
                }
            }
        }
        Ok(Vec::new())
    }

    /// Adds all the given elements to the queue, except for any
    /// elements with elapsed time greater than the allowed maximum
    /// or having been processed before.
    /// All elements must have the same prior state (supplied in prev).
    pub fn extend<I>(&self, iter: I, prev: &Option<T>) -> Result<()>
    where
        I: IntoIterator<Item = ContextWrapper<T>>,
    {
        let vec: Vec<ContextWrapper<T>> = iter.into_iter().collect();
        if vec.is_empty() {
            return Ok(());
        }

        let vec = self.handle_extend_group(vec, prev)?;

        if vec.is_empty() {
            Ok(())
        } else {
            self.internal_extend(vec)
        }
    }

    /// Adds all the given elements to the queue, except for any
    /// elements with elapsed time greater than the allowed maximum
    /// or having been processed before.
    /// Records all the given elements *plus* the one to keep, as the children
    /// of the prior state (prev). Note that the one kept may have elapsed time
    /// greater than the allowed maximum, or may have been processed before.
    pub fn extend_keep_one(
        &self,
        mut elements: Vec<ContextWrapper<T>>,
        keep: &ContextWrapper<T>,
        prev: &Option<T>,
    ) -> Result<()> {
        elements.push(keep.clone());
        let mut vec = self.handle_extend_group(elements, prev)?;
        if let Some((ctx, ..)) = vec.last() {
            // It must always be the last element if present; handle_extend_group does not reorder.
            if ctx == keep.get() {
                vec.pop();
            }
            self.internal_extend(vec)
        } else {
            Ok(())
        }
    }

    /// Adds all the given elements to the queue, except for any
    /// elements with elapsed time greater than the allowed maximum
    /// or having been processed before. Of the ones remaining, the one with
    /// highest progress/lowest score is returned instead of being added to the queue.
    ///
    /// All elements must have the same prior state (supplied in prev).
    pub fn extend_get_best<I>(&self, iter: I, prev: &Option<T>) -> Result<Option<ContextWrapper<T>>>
    where
        I: IntoIterator<Item = ContextWrapper<T>>,
    {
        let vec: Vec<ContextWrapper<T>> = iter.into_iter().collect();
        if vec.is_empty() {
            return Ok(None);
        }

        let mut vec = self.handle_extend_group(vec, prev)?;

        if let Some((mi, _)) = vec
            .iter()
            .enumerate()
            .min_by_key(|(_, (_, p, score))| (W::NUM_CANON_LOCATIONS - p, score))
        {
            let (ctx, ..) = vec.swap_remove(mi);
            let BestTimes {
                elapsed,
                time_since_visit,
                ..
            } = self.db.get_best_times(&ctx).unwrap();
            self.internal_extend(vec)?;
            Ok(Some(ContextWrapper::with_times(
                ctx,
                elapsed,
                time_since_visit,
            )))
        } else {
            Ok(None)
        }
    }

    /// Like calling extend multiple times with different values for prev,
    /// but only enters the critical section once.
    pub fn extend_groups<I>(&self, iter: I) -> Result<()>
    where
        I: IntoIterator<Item = (Vec<ContextWrapper<T>>, Option<T>)>,
    {
        let mut vec = Vec::new();
        for (v, prev) in iter {
            vec.extend(self.handle_extend_group(v, &prev)?);
        }
        self.internal_extend(vec)
    }

    fn handle_extend_group(
        &self,
        mut vec: Vec<ContextWrapper<T>>,
        prev: &Option<T>,
    ) -> Result<Vec<(T, usize, Score<'w, W, T>)>> {
        let mut iskips = 0;
        let keeps = self.db.record_many(&mut vec, prev)?;
        debug_assert!(vec.len() == keeps.len());
        // TODO: Is it inefficient to deconstruct the wrapper here,
        // and then reconstruct the wrapper for the one element we keep,
        // or can we extract it first somehow?
        let vec: Vec<(T, usize, Score<'w, W, T>)> = vec
            .into_iter()
            .zip(keeps.into_iter())
            .filter_map(|(el, keep)| {
                if self.world.won(el.get()) {
                    None
                } else if let Some(score) = keep {
                    if el.elapsed() > self.db.max_time()
                        || self.db.metric().total_estimate_from_score(score) > self.db.max_time()
                    {
                        iskips += 1;
                        None
                    } else {
                        let progress = el.get().count_visits();
                        Some((el.into_inner(), progress, score))
                    }
                } else {
                    None
                }
            })
            .collect();

        self.iskips.fetch_add(iskips, Ordering::Release);
        Ok(vec)
    }

    fn internal_extend(&self, vec: Vec<(T, usize, Score<'w, W, T>)>) -> Result<()>
    where
        T: Ctx<World = W>,
    {
        let mut evicted = None;
        let start: Instant;
        {
            let mut queue = self.queue.lock().unwrap();
            start = Instant::now();
            let len = queue.len();
            if len + vec.len() > self.capacity {
                evicted = Some(Self::evict_internal(
                    &mut queue,
                    std::cmp::min(
                        std::cmp::max(len + vec.len() - self.capacity, self.min_evictions),
                        std::cmp::min(self.max_evictions, (len / 4) * 3),
                    ),
                ));
            }
            queue.extend(vec);
        }
        // Without the lock (but still blocking the extend op in this thread)
        if let Some(ev) = evicted {
            let len = ev.len();
            log::trace!(
                "extend+evict took {:?} with the lock, got {} elements",
                start.elapsed(),
                len
            );
            if !ev.is_empty() {
                self.evict_to_db(ev, "extend")?;
            }
        }

        Ok(())
    }

    pub fn db_cleanup(&self, batch_size: usize, exit_signal: &AtomicBool) -> Result<()> {
        self.db.cleanup(batch_size, exit_signal)
    }

    pub fn skip_stats(&self) -> (usize, usize, usize, usize) {
        let (iskips, pskips, dup_iskips, dup_pskips) = self.db.skip_stats();
        (
            self.iskips.load(Ordering::Acquire) + iskips,
            self.pskips.load(Ordering::Relaxed) + pskips,
            dup_iskips,
            dup_pskips,
        )
    }

    pub fn print_queue_histogram(&self) {
        let queue = self.queue.lock().unwrap();
        if queue.is_empty() {
            println!("Queue is empty, no graph to print");
            return;
        }
        let mut progresses = Vec::new();
        let mut prog_score = Vec::new();
        let mut prog_estimates = Vec::new();
        for (progress, _, score) in queue.iter() {
            let primary = self.db.metric().score_primary(*score) as f64;
            let total_estimate = self.db.metric().total_estimate_from_score(*score) as f64;
            let progress = progress as f64;
            progresses.push(progress);
            prog_score.push((progress, primary));
            prog_estimates.push((progress, total_estimate));
        }
        let queue_len = queue.len();
        let queue_caps = queue.bucket_capacities();
        let mut processed: Vec<f64> = Vec::new();
        for (progress, x) in self.processed_counts.iter().enumerate() {
            let progress = progress as f64;
            processed.extend(std::iter::repeat(progress).take(x.swap(0, Ordering::AcqRel)));
        }
        // unlock
        drop(queue);

        let h = Histogram::from_slice(
            progresses.as_slice(),
            HistogramBins::Count(W::NUM_CANON_LOCATIONS),
        );
        let v = ContinuousView::new()
            .add(h)
            .x_label("progress")
            .x_range(-1., 1. + W::NUM_CANON_LOCATIONS as f64);
        let cap: usize = queue_caps.into_iter().sum();
        println!(
            "Current heap contents (len={}, total capacity={}):\n{}",
            queue_len,
            cap,
            Page::single(&v).dimensions(90, 10).to_text().unwrap(),
        );

        let p = Plot::new(prog_score).point_style(PointStyle::new().marker(PointMarker::Circle));
        let v = ContinuousView::new()
            .add(p)
            .x_label("progress")
            .y_label("score")
            .x_range(-1., 1. + W::NUM_CANON_LOCATIONS as f64);
        println!(
            "Heap scores by progress level:\n{}",
            Page::single(&v).dimensions(90, 10).to_text().unwrap()
        );

        // We can avoid an extra graph if the score is just the total estimate only
        if std::mem::size_of::<Score<'w, W, T>>() > std::mem::size_of::<u32>() {
            let p = Plot::new(prog_estimates)
                .point_style(PointStyle::new().marker(PointMarker::Circle));
            let v = ContinuousView::new()
                .add(p)
                .x_label("progress")
                .y_label("total estimate")
                .x_range(-1., 1. + W::NUM_CANON_LOCATIONS as f64);
            println!(
                "Heap total estimates by progress level:\n{}",
                Page::single(&v).dimensions(90, 10).to_text().unwrap()
            );
        }

        let h = Histogram::from_slice(
            processed.as_slice(),
            HistogramBins::Count(W::NUM_CANON_LOCATIONS),
        );
        let v = ContinuousView::new()
            .add(h)
            .x_label("progress")
            .x_range(-1., 1. + W::NUM_CANON_LOCATIONS as f64);
        println!(
            "States checked since last time:\n{}",
            Page::single(&v).dimensions(90, 10).to_text().unwrap(),
        );
    }
}
