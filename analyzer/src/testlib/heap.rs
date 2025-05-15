use crate::context::*;
use crate::heap::HeapElement;
use crate::CommonHasher;
use lru::LruCache;
use std::collections::BinaryHeap;
use std::num::NonZeroUsize;

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
        let start = std::time::Instant::now();
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
}
