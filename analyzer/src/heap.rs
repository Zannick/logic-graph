extern crate plotlib;

use crate::context::*;
use crate::db::HeapDB;
use crate::CommonHasher;
use lru::LruCache;
use plotlib::page::Page;
use plotlib::repr::{Histogram, HistogramBins, Plot};
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use priority_queue::DoublePriorityQueue;
use sort_by_derive::SortBy;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

#[derive(Debug, SortBy)]
pub(crate) struct HeapElement<T: Ctx> {
    #[sort_by]
    pub(crate) score: i32,
    pub(crate) el: ContextWrapper<T>,
}

/// A wrapper around a BinaryHeap of ContextWrapper<T> wherein:
/// * items are sorted by a "score" combination of progress and elapsed time
///   (controlled by the ContextWrapper object)
/// * a threshold of elapsed time can be set to make the heap ignore
///   items that have surpassed the elapsed time.
pub struct LimitedHeap<T: Ctx> {
    max_time: i32,
    heap: BinaryHeap<HeapElement<T>>,
    states_seen: LruCache<T, i32, CommonHasher>,
    scale_factor: i32,
    iskips: i32,
    pskips: i32,
    dup_skips: u32,
    dup_pskips: i32,
    last_clean: i32,
}

impl<T: Ctx> Default for LimitedHeap<T> {
    fn default() -> LimitedHeap<T> {
        LimitedHeap::new()
    }
}

impl<T: Ctx> LimitedHeap<T> {
    pub fn new() -> LimitedHeap<T> {
        LimitedHeap {
            max_time: i32::MAX,
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

    pub fn scale_factor(&self) -> i32 {
        self.scale_factor
    }

    pub fn set_scale_factor(&mut self, factor: i32) {
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

    pub fn max_time(&self) -> i32 {
        self.max_time
    }

    pub fn set_max_time(&mut self, max_time: i32) {
        self.max_time = core::cmp::min(self.max_time, max_time);
    }

    pub fn set_lenient_max_time(&mut self, max_time: i32) {
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
                score: el.score(self.scale_factor),
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
                score: el.score(factor),
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
                    score: c.score(self.scale_factor),
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

    pub fn stats(&self) -> (i32, i32, u32, i32) {
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
            .map(|c| (c.el.elapsed().into(), c.el.score(self.scale_factor).into()))
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

pub struct RocksBackedQueue<T: Ctx> {
    queue: Mutex<DoublePriorityQueue<ContextWrapper<T>, i32, CommonHasher>>,
    db: HeapDB<T>,
    capacity: AtomicUsize,
    iskips: AtomicUsize,
    pskips: AtomicUsize,
    max_db_priority: AtomicI32,
    min_evictions: usize,
    min_reshuffle: usize,
}

impl<T: Ctx> RocksBackedQueue<T> {
    pub fn new<P>(
        db_path: P,
        initial_max_time: i32,
        max_capacity: usize,
        min_evictions: usize,
        min_reshuffle: usize,
    ) -> Result<RocksBackedQueue<T>, String>
    where
        P: AsRef<Path>,
    {
        Ok(RocksBackedQueue {
            queue: Mutex::new(DoublePriorityQueue::with_capacity_and_hasher(
                max_capacity,
                CommonHasher::default(),
            )),
            db: HeapDB::open(db_path, initial_max_time)?,
            capacity: max_capacity.into(),
            iskips: 0.into(),
            pskips: 0.into(),
            max_db_priority: i32::MIN.into(),
            min_evictions,
            min_reshuffle,
        })
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

    pub fn db_best(&self) -> i32 {
        self.max_db_priority.load(Ordering::Acquire)
    }

    /// Returns whether the underlying queue and db are actually empty.
    /// Even if this returns false, attempting to peek or pop may produce None.
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty() && self.db.len() == 0
    }

    pub fn max_time(&self) -> i32 {
        self.db.max_time()
    }

    pub fn set_max_time(&self, max_time: i32) {
        self.db.set_max_time(max_time);
    }

    pub fn set_lenient_max_time(&self, max_time: i32) {
        self.db.set_lenient_max_time(max_time);
    }

    pub fn scale_factor(&self) -> i32 {
        self.db.scale_factor()
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

        let priority = el.score(self.db.scale_factor());
        let mut evicted = None;
        {
            let mut queue = self.queue.lock().unwrap();

            if queue.len() == self.capacity.load(Ordering::Acquire) {
                let (ctx, &p_min) = queue
                    .peek_min()
                    .ok_or("queue at capacity with no elements")?;
                if priority < p_min || (priority == p_min && el.elapsed() >= ctx.elapsed()) {
                    // Lower priority (or equal but later), evict the new item immediately
                    self.db.push(el)?;
                    self.max_db_priority.fetch_max(priority, Ordering::Release);
                } else {
                    let max_evictions = (queue.len() / 4) * 3;
                    // New item is better, evict some old_items.
                    evicted = Some(Self::evict_until(
                        &mut queue,
                        priority,
                        self.min_evictions,
                        max_evictions,
                    ));
                    debug_assert!(
                        priority == el.score(self.db.scale_factor()),
                        "priority {} didn't match score {}",
                        priority,
                        el.score(self.db.scale_factor())
                    );
                    queue.push(el, priority);
                }
            } else {
                debug_assert!(
                    priority == el.score(self.db.scale_factor()),
                    "priority {} didn't match score {}",
                    priority,
                    el.score(self.db.scale_factor())
                );
                queue.push(el, priority);
            }
        }
        // Without the lock (but still blocking the push op in this thread)
        if let Some(ev) = evicted {
            println!("push+evict took {:?} with the lock", start.elapsed());
            let start = Instant::now();
            if !ev.is_empty() {
                let best = ev
                    .iter()
                    .map(|ctx| ctx.score(self.db.scale_factor()))
                    .max()
                    .unwrap();
                self.db.extend(ev, true)?;
                self.max_db_priority.fetch_max(best, Ordering::Release);
                println!("evict to db took {:?}", start.elapsed());
            }
        }

        Ok(())
    }

    /// Removes elements from the min end of the queue until we reach any of:
    /// an element above the given priority (it is kept in the queue),
    /// the other end of the queue, or a total of `max_evictions` elements.
    /// You may also specify a `min_evictions` to ensure that a certain amount of space is
    /// always cleared.
    fn evict_until(
        queue: &mut MutexGuard<DoublePriorityQueue<ContextWrapper<T>, i32, CommonHasher>>,
        priority: i32,
        min_evictions: usize,
        max_evictions: usize,
    ) -> Vec<ContextWrapper<T>> {
        let mut evicted = Vec::new();
        while evicted.len() < max_evictions {
            if let Some((_, &prio)) = queue.peek_min() {
                if prio <= priority || evicted.len() < min_evictions {
                    evicted.push(queue.pop_min().unwrap().0);
                    continue;
                }
            }
            break;
        }
        evicted
    }

    /// Retrieves up to the given number of elements from the db and puts them in the queue.
    fn retrieve(
        queue: &mut MutexGuard<DoublePriorityQueue<ContextWrapper<T>, i32, CommonHasher>>,
        db: &HeapDB<T>,
        max_db_priority: &AtomicI32,
        num: usize,
    ) -> Result<(), String> {
        let start = Instant::now();
        queue.extend(db.retrieve(num)?.into_iter().map(|el| {
            let score = el.score(db.scale_factor());
            (el, score)
        }));
        println!("Retrieve from db took {:?}", start.elapsed());
        // the max priority in the db is probably now the min of this queue
        // or thereabouts
        max_db_priority.store(*queue.peek_min().unwrap().1, Ordering::Release);
        Ok(())
    }

    pub fn pop(&self) -> Result<Option<ContextWrapper<T>>, String> {
        let mut queue = self.queue.lock().unwrap();
        while !queue.is_empty() || !self.db.is_empty() {
            while let Some((_, &prio)) = queue.peek_max() {
                let db_prio = self.max_db_priority.load(Ordering::Acquire);
                // Only when we go a decent bit over
                if prio < db_prio * 101 / 100 {
                    let start = Instant::now();
                    let cap = self.capacity.load(Ordering::Acquire);
                    // Get a decent amount to refill
                    let num_to_restore = std::cmp::min(self.min_reshuffle, (cap - queue.len()) / 2);
                    let len = queue.len();
                    if cap - len < num_to_restore {
                        let evicted = Self::evict_until(
                            &mut queue,
                            prio,
                            self.min_evictions,
                            len + 2 * num_to_restore - cap,
                        );
                        println!("reshuffle:evict took {:?}", start.elapsed());
                        let s2 = Instant::now();

                        let best = evicted
                            .iter()
                            .map(|ctx| ctx.score(self.db.scale_factor()))
                            .max()
                            .unwrap();
                        self.db.extend(evicted, true)?;
                        self.max_db_priority.fetch_max(best, Ordering::Release);
                        println!("reshuffle:evict to db took {:?}", s2.elapsed());
                    };
                    Self::retrieve(&mut queue, &self.db, &self.max_db_priority, num_to_restore)?;
                    println!("Reshuffle took total {:?}", start.elapsed());
                    assert!(!queue.is_empty(), "Queue should have data after retrieve");
                }
                let (ctx, prio) = queue.pop_max().unwrap();
                debug_assert!(
                    prio == ctx.score(self.db.scale_factor()),
                    "priority {} didn't match score {}",
                    prio,
                    ctx.score(self.db.scale_factor())
                );
                if ctx.elapsed() > self.db.max_time() {
                    self.pskips.fetch_add(1, Ordering::Release);
                    continue;
                }
                if !self.db.remember_pop(&ctx)? {
                    continue;
                }
                return Ok(Some(ctx));
            }
            // Retrieve some from db
            Self::retrieve(
                &mut queue,
                &self.db,
                &self.max_db_priority,
                self.capacity.load(Ordering::Acquire) / 2,
            )?;
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
                if el.elapsed() > self.db.max_time() {
                    iskips += 1;
                    false
                } else {
                    true
                }
            })
            .collect();

        let keeps = self.db.remember_which(&vec)?;
        debug_assert!(vec.len() == keeps.len());
        let vec: Vec<(ContextWrapper<T>, i32)> = vec
            .into_iter()
            .zip(keeps.into_iter())
            .filter_map(|(el, keep)| {
                if keep {
                    let priority = el.score(self.db.scale_factor());
                    Some((el, priority))
                } else {
                    None
                }
            })
            .collect();

        let mut evicted = None;
        {
            let mut queue = self.queue.lock().unwrap();
            let cap = self.capacity.load(Ordering::Acquire);
            let len = queue.len();
            if len + vec.len() > cap {
                let max_evictions = (len / 4) * 3;
                let priority = vec.iter().max_by_key(|(_, p)| p).unwrap().1;
                evicted = Some(Self::evict_until(
                    &mut queue,
                    priority,
                    std::cmp::max(len + vec.len() - cap, self.min_evictions),
                    max_evictions,
                ));
            }
            queue.extend(vec);
        }
        // Without the lock (but still blocking the extend op in this thread)
        if let Some(ev) = evicted {
            println!("extend+evict took {:?} with the lock", start.elapsed());
            let start = Instant::now();
            if !ev.is_empty() {
                let best = ev
                    .iter()
                    .map(|ctx| ctx.score(self.db.scale_factor()))
                    .max()
                    .unwrap();
                self.db.extend(ev, true)?;
                self.max_db_priority.fetch_max(best, Ordering::Release);
                println!("evict to db took {:?}", start.elapsed());
            }
        }

        Ok(())
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
        let times: Vec<f64> = queue.iter().map(|c| c.0.elapsed().into()).collect();
        let time_scores: Vec<(f64, f64)> = queue
            .iter()
            .map(|c| {
                (
                    c.0.elapsed().into(),
                    c.0.score(self.db.scale_factor()).into(),
                )
            })
            .collect();
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
    }
}
