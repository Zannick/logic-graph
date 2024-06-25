use crate::CommonHasher;
use priority_queue::DoublePriorityQueue;
use std::hash::{BuildHasher, Hash};

/// A min priority queue that can only ever return a fixed number of elements.
/// Once that many elements have been popped, the queue cannot be used anymore and should be dropped.
///
/// If pushing would add more than the remaining number of elements, the element with max priority is evicted.
/// This could be the element being pushed.
///
/// Pushing an item that already exists in the queue changes its priority only.
pub struct LimitedPriorityQueue<I: Eq + Hash, P: Ord, H = CommonHasher> {
    queue: DoublePriorityQueue<I, P, H>,
    iters_left: usize,
}

impl<I: Eq + Hash, P: Clone + Ord> LimitedPriorityQueue<I, P, CommonHasher> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: DoublePriorityQueue::with_capacity_and_hasher(capacity, CommonHasher::default()),
            iters_left: capacity,
        }
    }

    pub fn with_limit(limit: usize) -> Self {
        Self {
            queue: DoublePriorityQueue::with_hasher(CommonHasher::default()),
            iters_left: limit,
        }
    }

    pub fn with_capacity_and_limit(capacity: usize, limit: usize) -> Self {
        Self {
            queue: DoublePriorityQueue::with_capacity_and_hasher(capacity, CommonHasher::default()),
            iters_left: limit,
        }
    }

    pub fn increase_limit(&mut self, add: usize) {
        self.iters_left += add;
    }
}

impl<I: Eq + Hash, P: Clone + Ord, H: BuildHasher> LimitedPriorityQueue<I, P, H> {
    pub fn with_limit_and_hasher(limit: usize, hasher: H) -> Self {
        Self {
            queue: DoublePriorityQueue::with_hasher(hasher),
            iters_left: limit,
        }
    }

    pub fn with_capacity_and_hasher(capacity: usize, hasher: H) -> Self {
        Self {
            queue: DoublePriorityQueue::with_capacity_and_hasher(capacity, hasher),
            iters_left: capacity,
        }
    }

    pub fn with_capacity_limit_and_hasher(capacity: usize, limit: usize, hasher: H) -> Self {
        Self {
            queue: DoublePriorityQueue::with_capacity_and_hasher(capacity, hasher),
            iters_left: limit,
        }
    }

    pub fn push(&mut self, item: I, priority: P) -> Option<P> {
        if self.queue.len() < self.iters_left {
            self.queue.push(item, priority)
        } else if let Some(p) = self.queue.change_priority(&item, priority.clone()) {
            Some(p)
        } else if let Some(max) = self.queue.peek_max() {
            if priority.clone() < *max.1 {
                self.queue.pop_max();
                self.queue.push(item, priority)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn pop(&mut self) -> Option<(I, P)> {
        self.iters_left = self.iters_left.saturating_sub(1);
        self.queue.pop_min()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn capacity_left(&self) -> usize {
        self.iters_left
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn is_expired(&self) -> bool {
        self.iters_left == 0
    }
}
