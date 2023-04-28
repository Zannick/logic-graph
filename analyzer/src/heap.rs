extern crate plotlib;

use crate::bucket::*;
use crate::context::*;
use crate::db::HeapDB;
use crate::estimates::ContextScorer;
use crate::steiner::*;
use crate::world::*;
use crate::CommonHasher;
use bucket_queue::BucketQueue;
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
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
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
        scale_factor * ctx.get().progress() * ctx.get().progress() 
        + (1 << 28) - ctx.elapsed()
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
    queue: Mutex<BucketQueue<Segment<ContextWrapper<T>, u32>>>,
    db: HeapDB<'w, W, T>,
    capacity: AtomicUsize,
    iskips: AtomicUsize,
    pskips: AtomicUsize,
    min_db_estimate: AtomicU32,
    min_evictions: usize,
    max_evictions: usize,
    min_reshuffle: usize,
    max_reshuffle: usize,
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
    ) -> Result<RocksBackedQueue<'w, W, T>, String>
    where
        P: AsRef<Path>,
    {
        let q = RocksBackedQueue {
            queue: Mutex::new(BucketQueue::new()),
            db: HeapDB::open(db_path, initial_max_time, world, startctx.get())?,
            capacity: max_capacity.into(),
            iskips: 0.into(),
            pskips: 0.into(),
            min_db_estimate: u32::MAX.into(),
            min_evictions,
            max_evictions,
            min_reshuffle,
            max_reshuffle,
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

    pub fn db_best(&self) -> u32 {
        self.min_db_estimate.load(Ordering::Acquire)
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
    pub fn push(&self, el: ContextWrapper<T>) -> Result<(), String> {
        let start = Instant::now();
        if el.elapsed() > self.db.max_time() {
            self.iskips.fetch_add(1, Ordering::Release);
            return Ok(());
        }
        if !self.db.remember_push(&el)? {
            return Ok(());
        }

        let est_complete = self.db.score(&el);
        if est_complete > self.db.max_time() {
            self.iskips.fetch_add(1, Ordering::Release);
            return Ok(());
        };
        let mut evicted = None;
        {
            let mut queue = self.queue.lock().unwrap();

            if queue.len() == self.capacity.load(Ordering::Acquire) {
                // compare to the last element, aka the MAX
                let (ctx, &p_max) = queue
                    .peek_max()
                    .ok_or("queue at capacity with no elements")?;
                if est_complete > p_max || (est_complete == p_max && el.elapsed() >= ctx.elapsed())
                {
                    // Lower priority (or equal but later), evict the new item immediately
                    self.db.push(el)?;
                    self.min_db_estimate
                        .fetch_min(est_complete, Ordering::Release);
                } else {
                    let max_evictions = std::cmp::min(self.max_evictions, (queue.len() / 4) * 3);
                    // New item is better, evict some old_items.
                    evicted = Some(Self::evict_until(
                        &mut queue,
                        est_complete,
                        self.min_evictions,
                        max_evictions,
                    ));
                    queue.push(el, 0, est_complete);
                }
            } else {
                queue.push(el, 0, est_complete);
            }
        }
        // Without the lock (but still blocking the push op in this thread)
        if let Some(ev) = evicted {
            println!("push+evict took {:?} with the lock", start.elapsed());
            let start = Instant::now();
            if !ev.is_empty() {
                let best = ev.iter().map(|ctx| self.db.score(ctx)).min().unwrap();
                self.db.extend(ev, true)?;
                self.min_db_estimate.fetch_min(best, Ordering::Release);
                self.evictions.fetch_add(1, Ordering::Release);
                println!("evict to db took {:?}", start.elapsed());
                println!("{}", self.db.get_memory_usage_stats().unwrap());
            }
        }

        Ok(())
    }

    /// Removes elements from the max end of the queue until we reach any of:
    /// an element above the given priority (it is kept in the queue),
    /// the other end of the queue, or a total of `max_evictions` elements.
    /// You may also specify a `min_evictions` to ensure that a certain amount of space is
    /// always cleared.
    fn evict_until(
        queue: &mut MutexGuard<BucketQueue<Segment<ContextWrapper<T>, u32>>>,
        el_estimate: u32,
        min_evictions: usize,
        max_evictions: usize,
    ) -> Vec<ContextWrapper<T>> {
        let mut evicted = Vec::new();
        while evicted.len() < max_evictions {
            if let Some((_, &est_completion)) = queue.peek_max() {
                if est_completion >= el_estimate || evicted.len() < min_evictions {
                    evicted.push(queue.pop_max().unwrap().0);
                    continue;
                }
            }
            break;
        }
        evicted
    }

    /// Retrieves up to the given number of elements from the db.
    fn retrieve(
        db: &HeapDB<W, T>,
        max_db_estimate: &AtomicU32,
        num: usize,
    ) -> Result<Vec<(ContextWrapper<T>, u32)>, String> {
        println!(
            "Beginning retrieve of {} entries, we have {} in the db",
            num,
            db.len()
        );
        let start = Instant::now();
        let res: Vec<_> = db
            .retrieve(max_db_estimate.load(Ordering::Acquire), num)?
            .into_iter()
            .map(|el| {
                let score = db.score(&el);
                (el, score)
            })
            .collect();
        println!("Retrieve from db took {:?}", start.elapsed());
        println!("{}", db.get_memory_usage_stats().unwrap());
        // the min priority in the db is probably now the max of this result, or thereabouts
        // which should be the last element
        if let Some(el) = res.last() {
            max_db_estimate.store(el.1, Ordering::Release);
        } else {
            max_db_estimate.store(u32::MAX, Ordering::Release);
        }
        Ok(res)
    }

    pub fn pop(&self) -> Result<Option<ContextWrapper<T>>, String> {
        let mut queue = self.queue.lock().unwrap();
        while !queue.is_empty() || !self.db.is_empty() {
            while let Some((_, &est_completion)) = queue.peek_min() {
                let db_best = self.min_db_estimate.load(Ordering::Acquire);
                // Only when we go a decent bit over
                if !self.db.is_empty()
                    && est_completion > db_best * 101 / 100
                    && !self.retrieving.fetch_or(true, Ordering::AcqRel)
                {
                    let start = Instant::now();
                    let cap = self.capacity.load(Ordering::Acquire);
                    // Get a decent amount to refill
                    let num_to_restore = std::cmp::max(
                        self.min_reshuffle,
                        std::cmp::min(self.max_reshuffle, (cap - queue.len()) / 2),
                    );
                    let len = queue.len();
                    if cap - len < num_to_restore {
                        let evicted = Self::evict_until(
                            &mut queue,
                            est_completion,
                            self.min_evictions,
                            len + 2 * num_to_restore - cap,
                        );
                        println!("reshuffle:evict took {:?}", start.elapsed());
                        drop(queue);
                        let s2 = Instant::now();

                        let best = evicted.iter().map(|ctx| self.db.score(ctx)).min().unwrap();
                        self.db.extend(evicted, true)?;
                        self.min_db_estimate.fetch_min(best, Ordering::Release);
                        self.evictions.fetch_add(1, Ordering::Release);
                        println!("reshuffle:evict to db took {:?}", s2.elapsed());
                    } else {
                        drop(queue);
                    }
                    let res = Self::retrieve(&self.db, &self.min_db_estimate, num_to_restore)?;
                    self.retrievals.fetch_add(1, Ordering::Release);
                    println!("Reshuffle took total {:?}", start.elapsed());
                    queue = self.queue.lock().unwrap();
                    queue.extend(res.into_iter().map(|(b, p)| (b, 0, p)));
                    assert!(!queue.is_empty(), "Queue should have data after retrieve");
                    self.retrieving.store(false, Ordering::Release);
                }
                let (ctx, el_estimate) = queue.pop_min().unwrap();
                debug_assert!(
                    el_estimate == self.db.score(&ctx),
                    "stored estimate {} didn't match score {}",
                    el_estimate,
                    self.db.score(&ctx)
                );
                let max_time = self.db.max_time();
                if ctx.elapsed() > max_time || self.db.score(&ctx) > max_time {
                    self.pskips.fetch_add(1, Ordering::Release);
                    continue;
                }
                if !self.db.remember_pop(&ctx)? {
                    continue;
                }
                return Ok(Some(ctx));
            }
            // Retrieve some from db
            if !self.db.is_empty() {
                if !self.retrieving.fetch_or(true, Ordering::AcqRel) {
                    queue = self.do_retrieve_and_insert(queue)?;
                    self.retrieving.store(false, Ordering::Release);
                } else {
                    return self.db.pop(None).map_err(|e| e.message);
                }
            }
        }
        Ok(None)
    }

    fn do_retrieve_and_insert<'a>(
        &'a self,
        mut queue: MutexGuard<'a, BucketQueue<Segment<ContextWrapper<T>, u32>>>,
    ) -> Result<MutexGuard<'a, BucketQueue<Segment<ContextWrapper<T>, u32>>>, String>
    {
        let start = Instant::now();
        let num_to_restore = std::cmp::max(
            self.min_reshuffle,
            std::cmp::min(
                self.max_reshuffle,
                (self.capacity.load(Ordering::Acquire) - queue.len()) / 2,
            ),
        );
        drop(queue);
        let res = Self::retrieve(&self.db, &self.min_db_estimate, num_to_restore)?;
        queue = self.queue.lock().unwrap();
        queue.extend(res.into_iter().map(|(b, p)| (b, 0, p)));
        self.retrievals.fetch_add(1, Ordering::Release);
        println!("Retrieval took total {:?}", start.elapsed());
        Ok(queue)
    }

    pub fn pop_max_estimate(&self) -> Result<Option<ContextWrapper<T>>, String> {
        let mut queue = self.queue.lock().unwrap();
        while !queue.is_empty() || !self.db.is_empty() {
            while let Some((ctx, el_estimate)) = queue.pop_max() {
                debug_assert!(
                    el_estimate == self.db.score(&ctx),
                    "stored estimate {} didn't match score {}",
                    el_estimate,
                    self.db.score(&ctx)
                );
                let max_time = self.db.max_time();
                if ctx.elapsed() > max_time || self.db.score(&ctx) > max_time {
                    self.pskips.fetch_add(1, Ordering::Release);
                    continue;
                }
                if !self.db.remember_pop(&ctx)? {
                    continue;
                }
                return Ok(Some(ctx));
            }
            // Retrieve some from db
            if !self.db.is_empty() {
                if !self.retrieving.fetch_or(true, Ordering::AcqRel) {
                    queue = self.do_retrieve_and_insert(queue)?;
                    self.retrieving.store(false, Ordering::Release);
                } else {
                    return self.db.pop(None).map_err(|e| e.message);
                }
            }
        }
        Ok(None)
    }

    /// Adds all the given elements to the queue, except for any
    /// elements with elapsed time greater than the allowed maximum
    /// or having been seen before with a smaller elapsed time.
    pub fn extend<I>(&self, iter: I) -> Result<(), String>
    where
        I: IntoIterator<Item = ContextWrapper<T>>,
    {
        let mut iskips = 0;
        let start = Instant::now();
        let vec: Vec<ContextWrapper<T>> = iter
            .into_iter()
            .filter(|el| {
                if el.elapsed() > self.db.max_time() || self.db.score(&el) > self.db.max_time() {
                    iskips += 1;
                    false
                } else {
                    true
                }
            })
            .collect();
        if vec.is_empty() {
            self.iskips.fetch_add(iskips, Ordering::Release);
            return Ok(());
        }

        let keeps = self.db.remember_which(&vec)?;
        debug_assert!(vec.len() == keeps.len());
        let vec: Vec<(ContextWrapper<T>, u32)> = vec
            .into_iter()
            .zip(keeps.into_iter())
            .filter_map(|(el, keep)| {
                if keep {
                    let priority = self.db.score(&el);
                    Some((el, priority))
                } else {
                    None
                }
            })
            .collect();
        if vec.is_empty() {
            self.iskips.fetch_add(iskips, Ordering::Release);
            return Ok(());
        }

        let mut evicted = None;
        {
            let mut queue = self.queue.lock().unwrap();
            let cap = self.capacity.load(Ordering::Acquire);
            let len = queue.len();
            if len + vec.len() > cap {
                let max_evictions = std::cmp::min(self.max_evictions, (len / 4) * 3);
                let el_estimate = vec.iter().min_by_key(|(_, p)| p).unwrap().1;
                evicted = Some(Self::evict_until(
                    &mut queue,
                    el_estimate,
                    std::cmp::max(len + vec.len() - cap, self.min_evictions),
                    max_evictions,
                ));
            }
            queue.extend(vec.into_iter().map(|(b, p)| (b, 0, p)));
        }
        // Without the lock (but still blocking the extend op in this thread)
        if let Some(ev) = evicted {
            println!("extend+evict took {:?} with the lock", start.elapsed());
            let start = Instant::now();
            if !ev.is_empty() {
                let best = ev.iter().map(|ctx| self.db.score(ctx)).min().unwrap();
                self.db.extend(ev, true)?;
                self.min_db_estimate.fetch_min(best, Ordering::Release);
                self.evictions.fetch_add(1, Ordering::Release);
                println!(
                    "evict to db took {:?}, db now has {}",
                    start.elapsed(),
                    self.db.len()
                );
                println!("{}", self.db.get_memory_usage_stats().unwrap());
            }
        }

        self.iskips.fetch_add(iskips, Ordering::Release);
        Ok(())
    }

    pub fn db_cleanup(&self, batch_size: usize) -> Result<(), String> {
        Ok(self.db.cleanup(batch_size)?)
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
        let times: Vec<f64> = queue.iter().map(|c| c.0.elapsed().into()).collect();
        let mut time_scores = Vec::new();
        let mut time_progress = Vec::new();
        for c in queue.iter() {
            let elapsed: f64 = c.0.elapsed().into();
            let score: f64 = self.db.score(&c.0).into();
            let progress: f64 = c.0.get().progress().into();
            time_scores.push((elapsed, score));
            time_progress.push((elapsed, progress));
        }
        // unlock
        drop(queue);

        let h = Histogram::from_slice(times.as_slice(), HistogramBins::Count(70));
        let v = ContinuousView::new()
            .add(h)
            .x_label("elapsed time")
            .x_range(0., self.db.max_time().into());
        println!(
            "Current heap contents:\n{}",
            Page::single(&v).dimensions(90, 10).to_text().unwrap()
        );
        let p = Plot::new(time_scores).point_style(PointStyle::new().marker(PointMarker::Circle));
        let v = ContinuousView::new()
            .add(p)
            .x_label("elapsed time")
            .y_label("score")
            .x_range(0., self.db.max_time().into());
        println!(
            "Heap scores by time:\n{}",
            Page::single(&v).dimensions(90, 10).to_text().unwrap()
        );
        let p = Plot::new(time_progress).point_style(PointStyle::new().marker(PointMarker::Square));
        let v = ContinuousView::new()
            .add(p)
            .x_label("elapsed time")
            .y_label("progress")
            .x_range(0., self.db.max_time().into())
            .y_range(0., 100.);
        println!(
            "Heap progress by time:\n{}",
            Page::single(&v).dimensions(90, 10).to_text().unwrap()
        );
    }
}
