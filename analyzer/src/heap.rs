use crate::context::*;
use crate::CommonHasher;
use lru::LruCache;
use sort_by_derive::SortBy;
use std::collections::BinaryHeap;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::time::Instant;

#[derive(Debug, SortBy)]
struct HeapElement<T: Ctx> {
    #[sort_by]
    score: i32,
    el: ContextWrapper<T>,
}

/// A wrapper around a BinaryHeap of ContextWrapper<T> wherein:
/// * items are sorted by a "score" combination of progress and elapsed time
///   (controlled by the ContextWrapper object)
/// * a threshold of elapsed time can be set to make the heap ignore
///   items that have surpassed the elapsed time.
pub struct LimitedHeap<T: Ctx> {
    max_time: i32,
    heap: BinaryHeap<HeapElement<T>>,
    // TODO: replace with a faster hash
    // TODO: improve memory usage by condensing bool elements of the context
    // into bitflags. and/or use an LRU cache with a BIG size
    states_seen: LruCache<T, i32, CommonHasher>,
    scale_factor: i32,
    iskips: i32,
    pskips: i32,
    dup_skips: u32,
    dup_pskips: i32,
    last_clean: i32,
}

impl<T: Ctx> LimitedHeap<T> {
    pub fn new() -> LimitedHeap<T> {
        LimitedHeap {
            max_time: i32::MAX,
            heap: {
                let mut h = BinaryHeap::new();
                h.reserve(1048576);
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
            self.clean();
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
        if el.elapsed() <= self.max_time {
            self.heap.push(HeapElement {
                score: el.score(self.scale_factor),
                el,
            });
        } else {
            self.iskips += 1;
        }
    }

    pub fn see(&mut self, el: &ContextWrapper<T>) -> bool {
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

    pub fn clean(&mut self) {
        println!("Cleaning... {}", self.heap.len());
        let start = Instant::now();
        let mut theap = BinaryHeap::new();
        self.heap.shrink_to_fit();
        theap.reserve(std::cmp::min(1048576, self.heap.len()));
        for el in self.heap.drain() {
            if el.el.elapsed() <= self.max_time {
                if let Some(&time) = self.states_seen.get(el.el.get()) {
                    if el.el.elapsed() <= time {
                        theap.push(HeapElement {
                            score: el.el.score(self.scale_factor),
                            el: el.el,
                        });
                    } else {
                        self.dup_pskips += 1;
                    }
                } else {
                    theap.push(HeapElement {
                        score: el.el.score(self.scale_factor),
                        el: el.el,
                    });
                }
            } else {
                self.pskips += 1;
            }
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
}
