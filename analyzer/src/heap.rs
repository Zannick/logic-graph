use crate::context::*;
use sort_by_derive::SortBy;
use std::collections::BinaryHeap;
use std::fmt::Debug;

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
#[derive(Debug)]
pub struct LimitedHeap<T: Ctx> {
    max_time: i32,
    heap: BinaryHeap<HeapElement<T>>,
    iskips: i32,
    pskips: i32,
}

impl<T: Ctx> LimitedHeap<T> {
    pub fn new() -> LimitedHeap<T> {
        LimitedHeap {
            max_time: i32::MAX,
            heap: BinaryHeap::new(),
            iskips: 0,
            pskips: 0,
        }
    }

    /// Returns the actual number of elements in the heap.
    /// Iterating over the heap may not produce this many elements.
    pub fn len(&self) -> usize {
        self.heap.len()
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

    /// Pushes an element into the heap.
    /// If the element's elapsed time is greater than the allowed maximum, does nothing.
    pub fn push(&mut self, el: ContextWrapper<T>) {
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
    /// Will skip over any elements whose elapsed time is greater than the allowed maximum.
    pub fn pop(&mut self) -> Option<ContextWrapper<T>> {
        // Lazily clear when the max time is changed with elements in the heap
        while let Some(el) = self.heap.pop() {
            if el.el.elapsed() <= self.max_time {
                return Some(el.el);
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

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = ContextWrapper<T>>,
    {
        self.heap.extend(iter.into_iter().filter_map(|c| {
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
                Some(&e.el)
            } else {
                None
            }
        })
    }

    pub fn stats(&self) -> (i32, i32) {
        (self.iskips, self.pskips)
    }
}
