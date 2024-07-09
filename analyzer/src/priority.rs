use crate::CommonHasher;
use priority_queue::DoublePriorityQueue;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};

/// A min priority queue that can only ever return a fixed number of unique elements.
/// Once that many elements have been popped, the queue cannot be used anymore and should be dropped.
///
/// If pushing would add more than the remaining number of elements, the element with max priority is evicted.
/// This could be the element being pushed. (It will still be remembered at that priority.)
///
/// Pushing an item that has already been added to the queue will have no effect if the priority is equal or higher
/// to the minimum priority seen for that item's unique key previously. If the priority is lower and the element is
/// still in the queue, its priority will be updated without increasing the queue length.
pub struct LimitedPriorityQueue<I: Eq + Hash, J: Clone + Eq + Hash, P: Ord, H = CommonHasher> {
    min_pmap: HashMap<J, P, H>,
    queue: DoublePriorityQueue<I, P, H>,
    iters_left: usize,
}

impl<I: Eq + Hash, J: Clone + Eq + Hash, P: Clone + Ord>
    LimitedPriorityQueue<I, J, P, CommonHasher>
{
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            min_pmap: HashMap::with_capacity_and_hasher(capacity, CommonHasher::default()),
            queue: DoublePriorityQueue::with_capacity_and_hasher(capacity, CommonHasher::default()),
            iters_left: capacity,
        }
    }

    pub fn with_limit(limit: usize) -> Self {
        Self {
            min_pmap: HashMap::with_hasher(CommonHasher::default()),
            queue: DoublePriorityQueue::with_hasher(CommonHasher::default()),
            iters_left: limit,
        }
    }

    pub fn with_capacity_and_limit(capacity: usize, limit: usize) -> Self {
        Self {
            min_pmap: HashMap::with_capacity_and_hasher(capacity, CommonHasher::default()),
            queue: DoublePriorityQueue::with_capacity_and_hasher(capacity, CommonHasher::default()),
            iters_left: limit,
        }
    }

    pub fn increase_limit(&mut self, add: usize) {
        self.iters_left += add;
    }

    pub fn total_seen(&self) -> usize {
        self.min_pmap.len()
    }

    pub fn seen(&self, unique_key: &J) -> bool {
        self.min_pmap.contains_key(unique_key)
    }
}

impl<I: Eq + Hash, J: Clone + Eq + Hash, P: Clone + Ord, H: BuildHasher + Clone>
    LimitedPriorityQueue<I, J, P, H>
{
    pub fn with_limit_and_hasher(limit: usize, hasher: H) -> Self {
        Self {
            min_pmap: HashMap::with_hasher(hasher.clone()),
            queue: DoublePriorityQueue::with_hasher(hasher),
            iters_left: limit,
        }
    }

    pub fn with_capacity_and_hasher(capacity: usize, hasher: H) -> Self {
        Self {
            min_pmap: HashMap::with_capacity_and_hasher(capacity, hasher.clone()),
            queue: DoublePriorityQueue::with_capacity_and_hasher(capacity, hasher),
            iters_left: capacity,
        }
    }

    pub fn with_capacity_limit_and_hasher(capacity: usize, limit: usize, hasher: H) -> Self {
        Self {
            min_pmap: HashMap::with_capacity_and_hasher(capacity, hasher.clone()),
            queue: DoublePriorityQueue::with_capacity_and_hasher(capacity, hasher),
            iters_left: limit,
        }
    }

    fn push_to_queue(&mut self, item: I, priority: P) -> Option<P> {
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

    pub fn push(&mut self, item: I, unique_key: J, priority: P) -> Option<P> {
        if let Some(p) = self.min_pmap.get_mut(&unique_key) {
            if priority < *p {
                *p = priority.clone();
                self.push_to_queue(item, priority)
            } else {
                None
            }
        } else {
            self.min_pmap.insert(unique_key, priority.clone());
            self.push_to_queue(item, priority)
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

    pub fn into_unique_key_map(self) -> HashMap<J, P, H> {
        self.min_pmap
    }
}
