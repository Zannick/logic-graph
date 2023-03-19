use crate::context::*;
use sort_by_derive::SortBy;
use std::collections::{BinaryHeap, HashMap};
use std::fmt::Debug;
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
    states_seen: HashMap<T, i32>,
    iskips: i32,
    pskips: i32,
    dup_skips: u32,
    dup_pskips: i32,
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
            states_seen: HashMap::new(),
            iskips: 0,
            pskips: 0,
            dup_skips: 0,
            dup_pskips: 0,
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
        self.set_max_time(max_time + (max_time / 100))
    }

    /// Pushes an element into the heap.
    /// If the element's elapsed time is greater than the allowed maximum,
    /// or, the state has been previously seen with a lower elapsed time, does nothing.
    pub fn push(&mut self, el: ContextWrapper<T>) {
        if let Some(min) = self.states_seen.get_mut(el.get()) {
            if el.elapsed() < *min {
                *min = el.elapsed();
            } else {
                self.dup_skips += 1;
                return;
            }
        } else {
            self.states_seen.insert(el.get().clone(), el.elapsed());
        }
        if el.elapsed() <= self.max_time {
            self.heap.push(HeapElement {
                score: el.score(),
                el,
            });
        } else {
            self.iskips += 1;
        }
    }

    /// Returns the next element with the highest score, or None.
    /// Will skip over any elements whose elapsed time is greater than the allowed maximum,
    /// or whose elapsed time is greater than the minimum seen for that state.
    pub fn pop(&mut self) -> Option<ContextWrapper<T>> {
        // Lazily clear when the max time is changed with elements in the heap
        while let Some(el) = self.heap.pop() {
            if el.el.elapsed() <= self.max_time {
                if el.el.elapsed() <= *self.states_seen.get(el.el.get()).unwrap() {
                    return Some(el.el);
                } else {
                    self.dup_pskips += 1;
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
        theap.reserve(1048576);
        for el in self.heap.drain() {
            if el.el.elapsed() <= self.max_time {
                if el.el.elapsed() <= *self.states_seen.get(el.el.get()).unwrap() {
                    theap.push(el);
                } else {
                    self.dup_pskips += 1;
                }
            } else {
                self.pskips += 1;
            }
        }
        self.heap = theap;
        let done = start.elapsed();
        println!("... -> {}. Done in {:?}.", self.heap.len(), done);
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
                self.states_seen.insert(c.get().clone(), c.elapsed());
            }
            if c.elapsed() <= self.max_time {
                Some(HeapElement {
                    score: c.score(),
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
                if e.el.elapsed() <= *self.states_seen.get(e.el.get()).unwrap() {
                    Some(&e.el)
                } else {
                    None
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
