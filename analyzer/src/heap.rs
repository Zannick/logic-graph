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
}

impl<T: Ctx> LimitedHeap<T> {
    pub fn new() -> LimitedHeap<T> {
        LimitedHeap {
            max_time: i32::MAX,
            heap: BinaryHeap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn max_time(&self) -> i32 {
        self.max_time
    }

    pub fn set_max_time(&mut self, max_time: i32) {
        self.max_time = core::cmp::min(self.max_time, max_time);
    }

    pub fn push(&mut self, el: ContextWrapper<T>) {
        if el.elapsed() <= self.max_time {
            self.heap.push(HeapElement {
                score: el.score(),
                el,
            });
        }
    }

    pub fn pop(&mut self) -> Option<ContextWrapper<T>> {
        // Lazily clear when the max time is changed with elements in the heap
        while let Some(el) = self.heap.pop() {
            if el.el.elapsed() <= self.max_time {
                return Some(el.el);
            }
        }
        None
    }

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
}
