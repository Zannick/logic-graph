extern crate plotlib;

use crate::bucket::*;
use crate::context::*;
use crate::db::HeapDB;
use crate::estimates::ContextScorer;
use crate::solutions::SolutionCollector;
use crate::steiner::*;
use crate::world::*;
use crate::CommonHasher;
use anyhow::{anyhow, Result};
use bucket_queue::{Bucket, BucketQueue, Queue};
use lru::LruCache;
use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins, Plot};
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use sort_by_derive::SortBy;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

#[derive(Debug, SortBy)]
pub(crate) struct HeapElement<T: Ctx> {
    #[sort_by]
    pub(crate) score: u32,
    pub(crate) el: ContextWrapper<T>,
}

/// A wrapper around a BinaryHeap of ContextWrapper<T> wherein:
/// * items are sorted by a "score" combination of progress and elapsed time
///   (controlled by the ContextWrapper object)
/// * a threshold of elapsed time can be set to make the heap ignore
///   items that have surpassed the elapsed time.
pub struct LimitedHeap<T: Ctx> {
    max_time: u32,
    heap: BinaryHeap<HeapElement<T>>,
    states_seen: LruCache<T, u32, CommonHasher>,
    scale_factor: u32,
    iskips: u32,
    pskips: u32,
    dup_skips: u32,
    dup_pskips: u32,
    last_clean: u32,
}

impl<T: Ctx> Default for LimitedHeap<T> {
    fn default() -> LimitedHeap<T> {
        LimitedHeap::new()
    }
}

impl<T: Ctx> LimitedHeap<T> {
    fn score(ctx: &ContextWrapper<T>, scale_factor: u32) -> u32 {
        scale_factor * ctx.get().progress() * ctx.get().progress() + (1 << 28) - ctx.elapsed()
    }

    pub fn new() -> LimitedHeap<T> {
        LimitedHeap {
            max_time: u32::MAX,
            heap: {
                let mut h = BinaryHeap::new();
                h.reserve(2048);
                h
            },
            states_seen: LruCache::with_hasher(
                NonZeroUsize::new(1 << 23).unwrap(),
                CommonHasher::default(),
            ),
            scale_factor: 50,
            iskips: 0,
            pskips: 0,
            dup_skips: 0,
            dup_pskips: 0,
            last_clean: 0,
        }
    }

    /// Returns the actual number of elements in the heap.
    /// Iterating over the heap may not produce this many elements.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn seen(&self) -> usize {
        self.states_seen.len()
    }

    pub fn scale_factor(&self) -> u32 {
        self.scale_factor
    }

    pub fn set_scale_factor(&mut self, factor: u32) {
        self.scale_factor = factor;
        if !self.heap.is_empty() {
            println!("Recalculating scores");
            self.clean()
        }
    }

    /// Returns whether the underlying heap is actually empty.
    /// Attempting to peek or pop may produce None instead.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn max_time(&self) -> u32 {
        self.max_time
    }

    pub fn set_max_time(&mut self, max_time: u32) {
        self.max_time = core::cmp::min(self.max_time, max_time);
    }

    pub fn set_lenient_max_time(&mut self, max_time: u32) {
        self.set_max_time(max_time + (max_time / 128))
    }

    /// Pushes an element into the heap.
    /// If the element's elapsed time is greater than the allowed maximum,
    /// or, the state has been previously seen with an equal or lower elapsed time, does nothing.
    pub fn push(&mut self, el: ContextWrapper<T>) {
        if el.elapsed() <= self.max_time {
            if let Some(min) = self.states_seen.get_mut(el.get()) {
                if el.elapsed() < *min {
                    *min = el.elapsed();
                } else {
                    self.dup_skips += 1;
                    return;
                }
            } else {
                self.states_seen.push(el.get().clone(), el.elapsed());
            }
            self.heap.push(HeapElement {
                score: Self::score(&el, self.scale_factor),
                el,
            });
        } else {
            self.iskips += 1;
        }
    }

    pub fn see(&mut self, el: &ContextWrapper<T>) -> bool {
        if el.elapsed() <= self.max_time {
            if let Some(min) = self.states_seen.get_mut(el.get()) {
                if el.elapsed() < *min {
                    *min = el.elapsed();
                    true
                } else {
                    self.dup_skips += 1;
                    false
                }
            } else {
                self.states_seen.push(el.get().clone(), el.elapsed());
                true
            }
        } else {
            self.iskips += 1;
            false
        }
    }

    /// Returns the next element with the highest score, or None.
    /// Will skip over any elements whose elapsed time is greater than the allowed maximum,
    /// or whose elapsed time is greater than the minimum seen for that state.
    pub fn pop(&mut self) -> Option<ContextWrapper<T>> {
        // Lazily clear when the max time is changed with elements in the heap
        while let Some(el) = self.heap.pop() {
            if el.el.elapsed() <= self.max_time {
                if let Some(&time) = self.states_seen.get(el.el.get()) {
                    if el.el.elapsed() <= time {
                        return Some(el.el);
                    } else {
                        self.dup_pskips += 1;
                    }
                } else {
                    return Some(el.el);
                }
            } else {
                self.pskips += 1;
            }
        }
        None
    }

    /// Produces the actual first element of the heap.
    /// This may not be the element returned by pop().
    pub fn peek(&self) -> Option<&ContextWrapper<T>> {
        match self.heap.peek() {
            Some(el) => Some(&el.el),
            None => None,
        }
    }

    fn drain(&mut self) -> impl IntoIterator<Item = ContextWrapper<T>> + '_ {
        self.heap.drain().filter_map(|el| {
            if el.el.elapsed() <= self.max_time {
                if let Some(&time) = self.states_seen.get(el.el.get()) {
                    if el.el.elapsed() <= time {
                        Some(el.el)
                    } else {
                        self.dup_pskips += 1;
                        None
                    }
                } else {
                    Some(el.el)
                }
            } else {
                self.pskips += 1;
                None
            }
        })
    }

    pub fn clean(&mut self) {
        println!("Cleaning... {}", self.heap.len());
        let start = Instant::now();
        let mut theap = BinaryHeap::new();
        self.heap.shrink_to_fit();
        theap.reserve(std::cmp::min(1048576, self.heap.len()));
        let factor = self.scale_factor;
        for el in self.drain() {
            theap.push(HeapElement {
                score: Self::score(&el, factor),
                el,
            });
        }
        self.heap = theap;
        let done = start.elapsed();
        println!("... -> {}. Done in {:?}.", self.heap.len(), done);
        self.last_clean = self.max_time;
        self.print_histogram();
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = ContextWrapper<T>>,
    {
        self.heap.extend(iter.into_iter().filter_map(|c| {
            if let Some(min) = self.states_seen.get_mut(c.get()) {
                if c.elapsed() < *min {
                    *min = c.elapsed();
                } else {
                    self.dup_skips += 1;
                    return None;
                }
            } else {
                self.states_seen.push(c.get().clone(), c.elapsed());
            }
            if c.elapsed() <= self.max_time {
                Some(HeapElement {
                    score: Self::score(&c, self.scale_factor),
                    el: c,
                })
            } else {
                self.iskips += 1;
                None
            }
        }));
    }

    pub fn iter(&self) -> impl Iterator<Item = &ContextWrapper<T>> + '_ {
        self.heap.iter().filter_map(|e| {
            if e.el.elapsed() <= self.max_time {
                if let Some(&time) = self.states_seen.peek(e.el.get()) {
                    if e.el.elapsed() <= time {
                        Some(&e.el)
                    } else {
                        None
                    }
                } else {
                    Some(&e.el)
                }
            } else {
                None
            }
        })
    }

    pub fn stats(&self) -> (u32, u32, u32, u32) {
        (self.iskips, self.pskips, self.dup_skips, self.dup_pskips)
    }

    pub fn print_histogram(&self) {
        let times: Vec<f64> = self.heap.iter().map(|c| c.el.elapsed().into()).collect();
        let h = Histogram::from_slice(times.as_slice(), HistogramBins::Count(70));
        let v = ContinuousView::new()
            .add(h)
            .x_label("elapsed time")
            .x_range(0., self.max_time.into());
        println!(
            "Current heap contents:\n{}",
            Page::single(&v).dimensions(90, 10).to_text().unwrap()
        );
        let times: Vec<(f64, f64)> = self
            .heap
            .iter()
            .map(|c| {
                (
                    c.el.elapsed().into(),
                    Self::score(&c.el, self.scale_factor).into(),
                )
            })
            .collect();
        let p = Plot::new(times).point_style(PointStyle::new().marker(PointMarker::Circle));
        let v = ContinuousView::new()
            .add(p)
            .x_label("elapsed time")
            .y_label("score")
            .x_range(0., self.max_time.into());
        println!(
            "Heap scores by time:\n{}",
            Page::single(&v).dimensions(90, 10).to_text().unwrap()
        );
    }
}

pub struct RocksBackedQueue<'w, W: World, T: Ctx> {
    // TODO: Make the bucket element just T.
    queue: Mutex<BucketQueue<Segment<T, u32>>>,
    db: HeapDB<'w, W, T>,
    capacity: usize,
    iskips: AtomicUsize,
    pskips: AtomicUsize,
    min_evictions: usize,
    max_evictions: usize,
    min_reshuffle: usize,
    max_reshuffle: usize,
    max_possible_progress: usize,
    evictions: AtomicUsize,
    retrievals: AtomicUsize,
    retrieving: AtomicBool,
}

impl<'w, W, T, L, E> RocksBackedQueue<'w, W, T>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = E::Currency>,
{
    pub fn new<P>(
        db_path: P,
        world: &'w W,
        startctx: &ContextWrapper<T>,
        initial_max_time: u32,
        max_capacity: usize,
        min_evictions: usize,
        max_evictions: usize,
        min_reshuffle: usize,
        max_reshuffle: usize,
        solutions: Arc<Mutex<SolutionCollector<T>>>,
    ) -> Result<RocksBackedQueue<'w, W, T>, String>
    where
        P: AsRef<Path>,
    {
        let db = HeapDB::open(db_path, initial_max_time, world, startctx.get(), solutions)?;
        let max_possible_progress = db.scorer().remaining_visits(startctx.get());
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
            max_possible_progress,
            evictions: 0.into(),
            retrievals: 0.into(),
            retrieving: false.into(),
        };
        let s = Instant::now();
        let c = q.score(startctx);
        println!("Calculated estimate {} in {:?}", c, s.elapsed());
        let s = Instant::now();
        let c = q.score(startctx);
        println!("Calculated estimate again {} in {:?}", c, s.elapsed());
        Ok(q)
    }

    pub fn db(&self) -> &HeapDB<W, T> {
        &self.db
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

    pub fn heap_bests(&self) -> Vec<Option<u32>> {
        let queue = self.queue.lock().unwrap();
        queue.peek_all_buckets_min()
    }

    pub fn score(&self, ctx: &ContextWrapper<T>) -> u32 {
        self.db.score(ctx)
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
        if !self.db.record_one(&mut el, prev)? {
            return Ok(());
        }
        if el.elapsed() > self.db.max_time() {
            self.iskips.fetch_add(1, Ordering::Release);
            return Ok(());
        }

        let est_complete = self.db.score(&el);
        if est_complete > self.db.max_time() {
            self.iskips.fetch_add(1, Ordering::Release);
            return Ok(());
        };
        let progress = self.db.progress(el.get());
        let mut evicted = None;
        {
            let mut queue = self.queue.lock().unwrap();

            if queue.len() == self.capacity {
                // compare to the last element, aka the MAX
                let (ctx, &p_max) = queue
                    .peek_segment_max(progress)
                    .ok_or(anyhow!("queue at capacity with no elements"))?;
                let elapsed = self.db.get_best_elapsed(ctx)?;
                if est_complete > p_max || (est_complete == p_max && el.elapsed() >= elapsed) {
                    // Lower priority (or equal but later), evict the new item immediately
                    self.db.push_from_queue(el, est_complete)?;
                } else {
                    let evictions = std::cmp::min(
                        self.max_evictions,
                        std::cmp::max(self.min_evictions, queue.len() / 2),
                    );
                    // New item is better, evict some old_items.
                    evicted = Some(Self::evict_internal(&mut queue, evictions));
                    queue.push(el.into_inner(), progress, est_complete);
                }
            } else {
                queue.push(el.into_inner(), progress, est_complete);
            }
        }
        // Without the lock (but still blocking the push op in this thread)
        if let Some(ev) = evicted {
            println!("push+evict took {:?} with the lock", start.elapsed());
            if !ev.is_empty() {
                self.evict_to_db(ev, "push")?;
            }
        }

        Ok(())
    }

    fn evict_to_db(&self, ev: Vec<(T, u32)>, category: &str) -> Result<()> {
        let start = Instant::now();
        self.db.extend_from_queue(ev)?;
        self.evictions.fetch_add(1, Ordering::Release);
        println!("{}:evict to db took {:?}", category, start.elapsed());
        println!("{}", self.db.get_memory_usage_stats().unwrap());
        Ok(())
    }

    /// Removes elements from the max end of each segment in the queue until we reach
    /// the minimum desired evictions.
    fn evict_internal(
        queue: &mut MutexGuard<BucketQueue<Segment<T, u32>>>,
        min_evictions: usize,
    ) -> Vec<(T, u32)> {
        let mut evicted = queue.pop_likely_useless();
        if !evicted.is_empty() {
            println!("Evicted {} useless states", evicted.len());
        }
        let more = min_evictions.saturating_sub(evicted.len());
        if more > 0 {
            evicted.extend(queue.pop_max_proportionally(more));
        }
        queue.shrink_to_fit();
        evicted
    }

    /// Retrieves up to the given number of elements for the given segment from the db.
    fn retrieve(&self, segment: usize, num: usize) -> Result<Vec<(T, usize, u32)>> {
        println!(
            "Beginning retrieve of {} entries from segment {} and up, we have {} total in the db",
            num,
            segment,
            self.db.len()
        );
        let start = Instant::now();
        let res: Vec<_> = self
            .db
            .retrieve(segment, num)?
            .into_iter()
            .map(|(el, elapsed, est)| {
                let progress = self.db.progress(&el);
                (el, progress, elapsed + est)
            })
            .collect();

        println!("Retrieve from db took {:?}", start.elapsed());
        println!("{}", self.db.get_memory_usage_stats().unwrap());

        Ok(res)
    }

    pub fn pop(&self) -> Result<Option<ContextWrapper<T>>> {
        let mut queue = self.queue.lock().unwrap();
        while !queue.is_empty() || !self.db.is_empty() {
            while let Some((el, &est_completion)) = queue.peek_min() {
                let progress = self.db.progress(el);
                let db_best = self.db.db_best(progress);
                // Only when we go a decent bit over
                if !self.db.is_empty() && db_best < u32::MAX && est_completion > db_best * 101 / 100
                {
                    queue = self.maybe_reshuffle(progress, queue)?;
                }
                let (ctx, _) = queue.pop_min().unwrap();

                // Retrieve the best elapsed time.
                let elapsed = self.db.get_best_elapsed(&ctx)?;
                let est = self.db.estimated_remaining_time(&ctx);

                let max_time = self.db.max_time();
                if elapsed > max_time || elapsed + est > max_time {
                    self.pskips.fetch_add(1, Ordering::Release);
                    continue;
                }
                if self.db.remember_processed(&ctx)? {
                    self.db.count_duplicate();
                    continue;
                }
                return Ok(Some(ContextWrapper::with_elapsed(ctx, elapsed)));
            }
            // Retrieve some from db
            if !self.db.is_empty() {
                if !self.retrieving.fetch_or(true, Ordering::AcqRel) {
                    queue = self.do_retrieve_and_insert(0, queue)?;
                    self.retrieving.store(false, Ordering::Release);
                } else {
                    return Ok(self
                        .db
                        .pop(0)?
                        .map(|(el, elapsed)| ContextWrapper::with_elapsed(el, elapsed)));
                }
            }
        }
        Ok(None)
    }

    fn maybe_reshuffle<'a>(
        &'a self,
        progress: usize,
        mut queue: MutexGuard<'a, BucketQueue<Segment<T, u32>>>,
    ) -> Result<MutexGuard<BucketQueue<Segment<T, u32>>>> {
        if !self.retrieving.fetch_or(true, Ordering::AcqRel) {
            let start = Instant::now();
            // Get a decent amount to refill
            let num_to_restore = std::cmp::max(
                self.min_reshuffle,
                std::cmp::min(self.max_reshuffle, (self.capacity - queue.len()) / 2),
            );
            let len = queue.len();
            if self.capacity - len < num_to_restore {
                // evict at least twice that much.
                let evicted = Self::evict_internal(
                    &mut queue,
                    std::cmp::max(self.min_evictions, 2 * num_to_restore),
                );
                println!("reshuffle:evict took {:?}", start.elapsed());
                drop(queue);
                self.evict_to_db(evicted, "reshuffle")?;
            } else {
                drop(queue);
            }
            let res = self.retrieve(progress, num_to_restore)?;
            self.retrievals.fetch_add(1, Ordering::Release);
            println!("Reshuffle took total {:?}", start.elapsed());
            queue = self.queue.lock().unwrap();
            queue.extend(res);
            assert!(!queue.is_empty(), "Queue should have data after retrieve");
            self.retrieving.store(false, Ordering::Release);
        }
        Ok(queue)
    }

    fn do_retrieve_and_insert<'a>(
        &'a self,
        segment: usize,
        mut queue: MutexGuard<'a, BucketQueue<Segment<T, u32>>>,
    ) -> Result<MutexGuard<'a, BucketQueue<Segment<T, u32>>>> {
        let start = Instant::now();
        let num_to_restore = std::cmp::max(
            self.min_reshuffle,
            std::cmp::min(
                self.max_reshuffle,
                (self.capacity - queue.len()) / self.max_possible_progress,
            ),
        );
        drop(queue);
        let res = self.retrieve(segment, num_to_restore)?;
        queue = self.queue.lock().unwrap();
        queue.extend(res);
        self.retrievals.fetch_add(1, Ordering::Release);
        println!("Retrieval took total {:?}", start.elapsed());
        Ok(queue)
    }

    fn pop_special<F>(&self, n: usize, pop_func: F) -> Result<Vec<ContextWrapper<T>>>
    where
        F: Fn(&mut MutexGuard<BucketQueue<Segment<T, u32>>>) -> Option<(T, u32)>,
    {
        let mut vec = Vec::new();
        let mut queue = self.queue.lock().unwrap();
        while vec.len() < n && (!queue.is_empty() || !self.db.is_empty()) {
            while let Some((ctx, _)) = (pop_func)(&mut queue) {
                // Retrieve the best elapsed time.
                let elapsed = self.db.get_best_elapsed(&ctx)?;
                let est = self.db.estimated_remaining_time(&ctx);
                let max_time = self.db.max_time();
                if elapsed > max_time || elapsed + est > max_time {
                    self.pskips.fetch_add(1, Ordering::Release);
                    continue;
                }
                if self.db.remember_processed(&ctx)? {
                    self.db.count_duplicate();
                    continue;
                }
                vec.push(ContextWrapper::with_elapsed(ctx, elapsed));
                if vec.len() == n {
                    return Ok(vec);
                }
            }
            // Retrieve some from db
            if !self.db.is_empty() {
                if !self.retrieving.fetch_or(true, Ordering::AcqRel) {
                    queue = self.do_retrieve_and_insert(0, queue)?;
                    self.retrieving.store(false, Ordering::Release);
                } else if let Some((ctx, elapsed)) = self.db.pop(0)? {
                    vec.push(ContextWrapper::with_elapsed(ctx, elapsed));
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

    pub fn pop_local_minima(&self, n: usize) -> Result<Vec<ContextWrapper<T>>> {
        let mut vec = Vec::new();
        let mut queue = self.queue.lock().unwrap();
        if queue.is_empty() {
            return Ok(vec);
        }

        let min = queue.min_priority().unwrap();
        let max = queue.max_priority().unwrap();
        let next = queue
            .bucket_for_peeking(max)
            .map(|b| b.min_priority().copied())
            .flatten();
        let mut target = 0;
        for segment in (min..max).rev() {
            let prev = queue
                .bucket_for_peeking(segment)
                .map(|b| b.min_priority().copied())
                .flatten();
            if let (Some(lower), Some(higher)) = (prev, next) {
                if higher < lower {
                    target = segment + 1;
                    break;
                }
            }
        }

        while vec.len() < n && (!queue.is_empty() || !self.db.is_empty()) {
            while let Some((ctx, _)) = queue.pop_segment_min(target) {
                // Retrieve the best elapsed time.
                let elapsed = self.db.get_best_elapsed(&ctx)?;
                let est = self.db.estimated_remaining_time(&ctx);
                let max_time = self.db.max_time();
                if elapsed > max_time || elapsed + est > max_time {
                    self.pskips.fetch_add(1, Ordering::Release);
                    continue;
                }
                if self.db.remember_processed(&ctx)? {
                    self.db.count_duplicate();
                    continue;
                }
                vec.push(ContextWrapper::with_elapsed(ctx, elapsed));
                if vec.len() == n {
                    return Ok(vec);
                }
            }
        }
        Ok(vec)
    }

    pub fn pop_round_robin(&self) -> Result<Vec<ContextWrapper<T>>> {
        let mut queue = self.queue.lock().unwrap();
        let mut did_retrieve = false;
        while !queue.is_empty() || !self.db.is_empty() {
            if let Some(min) = queue.min_priority() {
                let max = queue.max_priority().unwrap();
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
                            let elapsed = self.db.get_best_elapsed(&ctx)?;
                            let est = self.db.estimated_remaining_time(&ctx);

                            // We won't actually use what is added here right away, unless we drop the element we just popped.
                            if !did_retrieve
                                && !self.db.is_empty()
                                && db_best < u32::MAX
                                && elapsed + est > db_best
                            {
                                diffs.push((segment, elapsed + est - db_best));
                            }

                            let max_time = self.db.max_time();
                            if elapsed > max_time || elapsed + est > max_time {
                                self.pskips.fetch_add(1, Ordering::Release);
                                continue;
                            }
                            if self.db.remember_processed(&ctx)? {
                                self.db.count_duplicate();
                                continue;
                            }

                            vec.push(ContextWrapper::with_elapsed(ctx, elapsed));
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
                                let elapsed = self.db.get_best_elapsed(&ctx)?;
                                let est = self.db.estimated_remaining_time(&ctx);
                                let max_time = self.db.max_time();
                                if elapsed > max_time || elapsed + est > max_time {
                                    self.pskips.fetch_add(1, Ordering::Release);
                                    continue;
                                }
                                if self.db.remember_processed(&ctx)? {
                                    self.db.count_duplicate();
                                    continue;
                                }
                                vec.push(ContextWrapper::with_elapsed(ctx, elapsed));
                                continue 'next;
                            }
                        }
                    }
                }
                // Max one retrieve per pop_round_robin
                if !did_retrieve {
                    if let Some((segment, _)) = diffs.into_iter().max_by_key(|p| p.1) {
                        let _queue = self.maybe_reshuffle(segment, queue)?;
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
                    } else if let Some((el, elapsed)) = self.db.pop(0)? {
                        return Ok(vec![ContextWrapper::with_elapsed(el, elapsed)]);
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
            return Ok(());
        }

        self.internal_extend(vec)
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
    ) -> Result<Vec<(T, usize, u32)>> {
        let mut iskips = 0;
        let keeps = self.db.record_many(&mut vec, prev)?;
        debug_assert!(vec.len() == keeps.len());
        let vec: Vec<(T, usize, u32)> = vec
            .into_iter()
            .zip(keeps.into_iter())
            .filter_map(|(el, keep)| {
                if el.elapsed() > self.db.max_time() || self.db.score(&el) > self.db.max_time() {
                    iskips += 1;
                    None
                } else if keep {
                    let priority = self.db.score(&el);
                    let progress = self.db.progress(el.get());
                    Some((el.into_inner(), progress, priority))
                } else {
                    None
                }
            })
            .collect();

        self.iskips.fetch_add(iskips, Ordering::Release);
        Ok(vec)
    }

    fn internal_extend(&self, vec: Vec<(T, usize, u32)>) -> Result<()>
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
            println!(
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

    pub fn db_cleanup(&self, batch_size: usize, exit_signal: &AtomicBool) -> Result<(), String> {
        Ok(self.db.cleanup(batch_size, exit_signal)?)
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
        let max_progress = <usize as TryInto<u32>>::try_into(self.max_possible_progress + 1)
            .unwrap()
            .into();
        let queue = self.queue.lock().unwrap();
        if queue.is_empty() {
            println!("Queue is empty, no graph to print");
            return;
        }
        let mut progresses = Vec::new();
        let mut prog_score = Vec::new();
        for (progress, _, score) in queue.iter() {
            let score: f64 = (*score).into();
            let progress: u32 = progress.try_into().unwrap();
            let progress: f64 = progress.into();
            progresses.push(progress);
            prog_score.push((progress, score));
        }
        // unlock
        let queue_buckets = queue.bucket_sizes();
        drop(queue);

        let h = Histogram::from_slice(
            progresses.as_slice(),
            HistogramBins::Count(self.max_possible_progress),
        );
        let v = ContinuousView::new().add(h).x_label("progress");
        println!(
            "Current heap contents:\n{}\n{:?}",
            Page::single(&v).dimensions(90, 10).to_text().unwrap(),
            queue_buckets
        );

        let p = Plot::new(prog_score).point_style(PointStyle::new().marker(PointMarker::Circle));
        let v = ContinuousView::new()
            .add(p)
            .x_label("progress")
            .y_label("score")
            .x_range(-1., max_progress);
        println!(
            "Heap scores by progress level:\n{}",
            Page::single(&v).dimensions(90, 10).to_text().unwrap()
        );
    }
}
