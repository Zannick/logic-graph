//! Uses bucket_queue to create a queue ordered by one priority measure
//! but segmented by another priority measure, such that callers can
//! retrieve the minimum (or maximum) of the SegmentedBucketQueue overall or the
//! minimum/maximum of a given segment

use crate::CommonHasher;
use bucket_queue::{Bucket, BucketQueue, Queue};
use log;
use priority_queue::DoublePriorityQueue;
use std::hash::Hash;

pub struct Segment<I, P>
where
    I: Hash + Eq,
    P: Ord,
{
    pq: DoublePriorityQueue<I, P, CommonHasher>,
}

impl<I, P> Bucket for Segment<I, P>
where
    I: Hash + Eq,
    P: Ord,
{
    type Item = I;

    fn new_bucket() -> Self {
        Segment {
            pq: DoublePriorityQueue::with_hasher(CommonHasher::default()),
        }
    }

    fn len_bucket(&self) -> usize {
        self.pq.len()
    }

    fn is_empty_bucket(&self) -> bool {
        self.pq.is_empty()
    }

    fn clear(&mut self) {
        self.pq.clear();
    }
}

pub trait SegmentBucket<P: Ord>: Bucket {
    fn push(&mut self, item: Self::Item, priority: P) -> Option<P>;
    fn pop_min(&mut self) -> Option<(Self::Item, P)>;
    fn pop_max(&mut self) -> Option<(Self::Item, P)>;

    fn peek_min(&self) -> Option<(&Self::Item, &P)>;
    fn peek_max(&self) -> Option<(&Self::Item, &P)>;
    fn min_priority(&self) -> Option<&P>;
    fn max_priority(&self) -> Option<&P>;

    fn shrink_to_fit(&mut self);
    fn iter(&self) -> priority_queue::core_iterators::Iter<Self::Item, P>
    where
        Self::Item: Hash + Eq;

    fn capacity(&self) -> usize;
}

impl<I, P> SegmentBucket<P> for Segment<I, P>
where
    I: Hash + Eq,
    P: Ord,
{
    fn push(&mut self, item: I, priority: P) -> Option<P> {
        self.pq.push(item, priority)
    }

    fn pop_min(&mut self) -> Option<(Self::Item, P)> {
        self.pq.pop_min()
    }

    fn pop_max(&mut self) -> Option<(Self::Item, P)> {
        self.pq.pop_max()
    }

    fn peek_min(&self) -> Option<(&Self::Item, &P)> {
        self.pq.peek_min()
    }

    fn peek_max(&self) -> Option<(&Self::Item, &P)> {
        self.pq.peek_max()
    }

    fn min_priority(&self) -> Option<&P> {
        self.pq.peek_min().map(|(_, p)| p)
    }

    fn max_priority(&self) -> Option<&P> {
        self.pq.peek_max().map(|(_, p)| p)
    }

    fn shrink_to_fit(&mut self) {
        self.pq.shrink_to_fit();
    }

    fn iter(&self) -> priority_queue::core_iterators::Iter<I, P>
    where
        Self::Item: Hash + Eq,
    {
        self.pq.iter()
    }

    fn capacity(&self) -> usize {
        self.pq.capacity()
    }
}

pub struct Iter<'b, Q, B, P>
where
    Q: SegmentedBucketQueue<'b, B, P>,
    B: SegmentBucket<P> + 'b,
    B::Item: Hash + Eq,
    P: Ord,
{
    q: &'b Q,
    iter: priority_queue::core_iterators::Iter<'b, B::Item, P>,
    bucket: usize,
    max: usize,
}

impl<'b, Q, B, P> Iterator for Iter<'b, Q, B, P>
where
    Q: SegmentedBucketQueue<'b, B, P>,
    B: SegmentBucket<P> + 'b,
    B::Item: Hash + Eq + 'b,
    P: Ord,
{
    type Item = (usize, &'b B::Item, &'b P);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.bucket > self.max {
                return None;
            }
            if let Some((b, p)) = self.iter.next() {
                return Some((self.bucket, b, p));
            }
            if self.bucket == self.max {
                return None;
            }
            while self.bucket < self.max {
                self.bucket += 1;
                if let Some(bucket) = self.q.bucket_for_peeking(self.bucket) {
                    self.iter = bucket.iter();
                    break;
                }
            }
        }
    }
}

pub trait SegmentedBucketQueue<'b, B: SegmentBucket<P> + 'b, P: Ord>: Queue<B> {
    fn push(&mut self, item: B::Item, segment: usize, priority: P) {
        if self
            .bucket_for_adding(segment)
            .push(item, priority)
            .is_some()
        {
            // We just updated a state's priority without adding 1
            // so we'd better reverse the index update.
            self.items_replaced(segment, 1, 0);
        }
    }

    fn extend<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = (B::Item, usize, P)>,
    {
        for (item, segment, priority) in items.into_iter() {
            self.push(item, segment, priority)
        }
    }

    fn pop_min(&mut self) -> Option<(B::Item, P)> {
        let segment = (self.min_priority()?..=self.max_priority()?)
            .filter(|&s| {
                self.bucket_for_peeking(s)
                    .is_some_and(|bucket| !bucket.is_empty_bucket())
            })
            .min_by_key(|&s| self.bucket_for_peeking(s).unwrap().min_priority())?;
        self.bucket_for_removing(segment)?.pop_min()
    }
    fn pop_max(&mut self) -> Option<(B::Item, P)> {
        let segment = (self.min_priority()?..=self.max_priority()?)
            .filter(|&s| {
                self.bucket_for_peeking(s)
                    .is_some_and(|bucket| !bucket.is_empty_bucket())
            })
            .max_by_key(|&s| self.bucket_for_peeking(s).unwrap().max_priority())?;
        self.bucket_for_removing(segment)?.pop_max()
    }

    fn peek_min(&'b self) -> Option<(&'b B::Item, &'b P)> {
        let segment = (self.min_priority()?..=self.max_priority()?)
            .filter(|&s| {
                self.bucket_for_peeking(s)
                    .is_some_and(|bucket| !bucket.is_empty_bucket())
            })
            .min_by_key(|&s| self.bucket_for_peeking(s).unwrap().min_priority())?;
        self.bucket_for_peeking(segment)?.peek_min()
    }
    fn peek_max(&'b self) -> Option<(&'b B::Item, &'b P)> {
        let segment = (self.min_priority()?..=self.max_priority()?)
            .filter(|&s| {
                self.bucket_for_peeking(s)
                    .is_some_and(|bucket| !bucket.is_empty_bucket())
            })
            .max_by_key(|&s| self.bucket_for_peeking(s).unwrap().max_priority())?;
        self.bucket_for_peeking(segment)?.peek_max()
    }

    fn peek_segment_max(&'b self, min_segment: usize) -> Option<(&'b B::Item, &'b P)> {
        for segment in min_segment..=self.max_priority()? {
            if let Some(b) = self.bucket_for_peeking(segment) {
                if let Some(x) = b.peek_max() {
                    return Some(x);
                }
            }
        }
        None
    }

    fn pop_segment_min(&mut self, min_segment: usize) -> Option<(B::Item, P)> {
        let min_segment = std::cmp::max(min_segment, self.min_priority()?);
        for segment in min_segment..=self.max_priority()? {
            if let Some(b) = self.bucket_for_replacing(segment) {
                if let Some(x) = b.pop_min() {
                    self.items_replaced(segment, 1, 0);
                    return Some(x);
                }
            }
        }
        None
    }

    fn pop_max_segment_min(&mut self) -> Option<(B::Item, P)> {
        let segment = self.max_priority()?;
        self.bucket_for_removing(segment)?.pop_min()
    }

    fn pop_min_segment_max(&mut self) -> Option<(B::Item, P)> {
        let segment = self.min_priority()?;
        self.bucket_for_removing(segment)?.pop_max()
    }

    fn peek_min_segment_min_priority(&'b self) -> Option<&'b P> {
        self.bucket_for_peeking(self.min_priority()?)?
            .peek_min()
            .map(|(_, p)| p)
    }

    fn peek_min_segment_max_priority(&'b self) -> Option<&'b P> {
        self.bucket_for_peeking(self.min_priority()?)?
            .peek_max()
            .map(|(_, p)| p)
    }

    fn peek_segment_priority_range(&'b self, segment: usize) -> Option<(&'b P, &'b P)> {
        let bucket = self.bucket_for_peeking(segment)?;
        if let Some((_, p)) = bucket.peek_min() {
            Some((p, bucket.peek_max().unwrap().1))
        } else {
            None
        }
    }

    /// Take n from the segment with the most.
    fn pop_n_from_largest_segment(&mut self, n: usize) -> Vec<(B::Item, P)> {
        if let Some(min) = self.min_priority() {
            let max = self.max_priority().unwrap();
            let mut vec = Vec::new();
            let segment = (min..=max)
                .into_iter()
                .max_by_key(|s| self.bucket_for_peeking(*s).map_or(0, |b| b.len_bucket()))
                .unwrap();
            if let Some(bucket) = self.bucket_for_replacing(segment) {
                while vec.len() < n {
                    if !bucket.is_empty_bucket() {
                        vec.push(bucket.pop_min().unwrap());
                    } else {
                        break;
                    }
                }
            }
            self.items_replaced(segment, vec.len(), 0);
            vec
        } else {
            Vec::new()
        }
    }

    /// More efficiently extracts all the items from all buckets with
    /// priorities above `keep_priority`.
    fn pop_all_with_priority(
        &mut self,
        keep_priority: P,
        max_segment: usize,
        max_pops: usize,
    ) -> Vec<(B::Item, P)> {
        if max_pops == 0 {
            Vec::new()
        } else if let Some(min) = self.min_priority() {
            let max = std::cmp::min(max_segment, self.max_priority().unwrap_or(min));
            let mut vec = Vec::new();
            for segment in min..=max {
                if self.bucket_for_peeking(segment).is_none() {
                    continue;
                }
                // We have to borrow and drop on each loop, since within the loop we need to borrow mutably
                while let Some(p) = self.bucket_for_peeking(segment).unwrap().max_priority() {
                    if *p > keep_priority {
                        vec.push(
                            self.bucket_for_removing(segment)
                                .unwrap()
                                .pop_max()
                                .unwrap(),
                        );
                        if vec.len() >= max_pops {
                            return vec;
                        }
                    } else {
                        break;
                    }
                }
            }
            vec
        } else {
            Vec::new()
        }
    }

    /// Round-robin eviction of `min_pops` elements across all segments.
    /// Will not completely empty any segment. Requires that the queue has at least
    /// `min_pops` elements, plus one for each non-empty segment.
    fn pop_max_round_robin(&mut self, min_pops: usize) -> Vec<(B::Item, P)> {
        if let Some(min) = self.min_priority() {
            let max = self.max_priority().unwrap();
            let mut vec = Vec::with_capacity(min_pops);
            let mut segment = min;
            assert!(
                self.len_queue() > min_pops + max - min,
                "Not enough in queue for {} pops: have min={} max={} len={}",
                min_pops,
                min,
                max,
                self.len_queue()
            );
            while vec.len() < min_pops {
                if let Some(bucket) = self.bucket_for_replacing(segment) {
                    if bucket.len_bucket() > 1 {
                        vec.push(bucket.pop_max().unwrap());
                        self.items_replaced(segment, 1, 0);
                    }
                }
                segment = if segment == max { min } else { segment + 1 };
            }
            vec
        } else {
            Vec::new()
        }
    }

    fn pop_max_proportionally(&mut self, min_pops: usize) -> Vec<(B::Item, P)> {
        if let Some(min) = self.min_priority() {
            let max = self.max_priority().unwrap();
            let mut vec = Vec::with_capacity(min_pops + max - min + 1);
            let factor = (self.len_queue() + min_pops - 1) / min_pops;
            assert!(factor > 1);
            for segment in min..=max {
                // pop 1/factor of each segment with at least that many elements.
                // and round up.
                if let Some(bucket) = self.bucket_for_replacing(segment) {
                    // This actually guarantees that we don't clear the list.
                    // min factor = 2, min len = 3, so we pop (3+1) / 2 = 2 elements
                    // higher factors leave even more.
                    if bucket.len_bucket() > factor {
                        let pops = (bucket.len_bucket() + factor - 1) / factor;
                        for _i in 0..pops {
                            vec.push(bucket.pop_max().unwrap());
                        }
                        self.items_replaced(segment, pops, 0);
                    }
                }
            }
            if vec.len() < min_pops {
                vec.extend(self.pop_max_round_robin(min_pops - vec.len()));
            }
            vec
        } else {
            Vec::new()
        }
    }

    /// Finds the lowest segment S and the highest corresponding segment S' below S
    /// where S-min > S'-max, and evicts all elements below S' with priority > S-max.
    fn pop_likely_useless(&mut self) -> Vec<(B::Item, P)>
    where
        P: Copy + std::fmt::Debug,
    {
        let vec = Vec::new();
        if let Some(min) = self.min_priority() {
            let max = self.max_priority().unwrap();
            for segment in (min + 2)..=max {
                let Some(bucket) = self.bucket_for_peeking(segment) else {
                    continue;
                };
                if bucket.len_bucket() < 2 {
                    continue;
                }
                let min_prio = bucket.min_priority().unwrap();
                let max_prio = bucket.max_priority().unwrap();

                for below in (min..segment).rev() {
                    let Some(blbucket) = self.bucket_for_peeking(below) else {
                        continue;
                    };
                    if blbucket.len_bucket() < 2 {
                        continue;
                    }
                    if blbucket.max_priority().unwrap() < min_prio {
                        let keep_priority = *max_prio;
                        log::debug!(
                            "Segment {}: {:?}..={:?} vs Segment {}: {:?}..={:?}, means we can evict \
                            from segment {} and below where score > {:?}",
                            below,
                            blbucket.min_priority().unwrap(),
                            blbucket.max_priority().unwrap(),
                            segment,
                            min_prio,
                            max_prio,
                            below,
                            keep_priority
                        );
                        return self.pop_all_with_priority(keep_priority, below, usize::MAX);
                    }
                }
            }
        }
        vec
    }

    fn shrink_to_fit(&mut self) {
        if let Some(max) = self.max_priority() {
            for i in 0..=max {
                if let Some(b) = self.bucket_for_replacing(i) {
                    b.shrink_to_fit();
                }
            }
        }
    }

    fn bucket_sizes(&self) -> Vec<usize> {
        if let Some(max) = self.max_priority() {
            (0..=max)
                .map(|s| {
                    self.bucket_for_peeking(s)
                        .map(|b| b.len_bucket())
                        .unwrap_or_default()
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn bucket_capacities(&self) -> Vec<usize> {
        if let Some(max) = self.max_priority() {
            (0..=max)
                .map(|s| {
                    self.bucket_for_peeking(s)
                        .map(|b| b.capacity())
                        .unwrap_or_default()
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn approx_num_buckets(&self) -> usize {
        if let Some(max) = self.max_priority() {
            max - self.min_priority().unwrap() + 1
        } else {
            0
        }
    }

    fn peek_all_buckets_min(&self) -> Vec<Option<P>>
    where
        P: Copy,
    {
        if let Some(max) = self.max_priority() {
            (0..=max)
                .map(|s| {
                    self.bucket_for_peeking(s)
                        .and_then(|b| b.peek_min())
                        .map(|m| *m.1)
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn iter(&'b self) -> Iter<'b, Self, B, P>
    where
        B::Item: Hash + Eq,
        Self: Sized,
    {
        let min = self.min_priority().unwrap_or_default();
        let max = self.max_priority().unwrap_or_default();
        Iter {
            q: self,
            iter: self.bucket_for_peeking(min).unwrap().iter(),
            bucket: min,
            max,
        }
    }
}

impl<'b, B: SegmentBucket<P> + 'b, P: Ord> SegmentedBucketQueue<'b, B, P> for BucketQueue<B> {}

#[cfg(test)]
mod test {
    use super::*;
    use bucket_queue::BucketQueue;

    #[test]
    fn pop() {
        let mut queue = BucketQueue::<Segment<&str, i8>>::new();
        queue.push("first", 0, 1);
        queue.push("third", 1, 2);
        queue.push("second", 1, 1);

        assert_eq!(queue.pop_min(), Some(("first", 1)));
        assert_eq!(queue.pop_min(), Some(("second", 1)));
        assert_eq!(queue.pop_min(), Some(("third", 2)));
        assert_eq!(queue.pop_min(), None);

        queue.push("first", 0, 1);
        queue.push("third", 1, 2);
        queue.push("second", 1, 1);
        assert_eq!(queue.pop_max(), Some(("third", 2)));
        assert_eq!(queue.pop_max(), Some(("second", 1)));
        assert_eq!(queue.pop_max(), Some(("first", 1)));
        assert_eq!(queue.pop_max(), None);
    }

    #[test]
    fn pop_min_segment() {
        let mut queue = BucketQueue::<Segment<&str, i8>>::new();
        queue.push("first", 0, 1);
        queue.push("third", 2, 1);
        queue.push("second", 0, 2);
        queue.push("fourth", 2, 3);

        assert_eq!(queue.pop_segment_min(1), Some(("third", 1)));
        assert_eq!(queue.pop_segment_min(2), Some(("fourth", 3)));
        assert_eq!(queue.pop_segment_min(1), None);
        assert_eq!(queue.len_queue(), 2);
    }

    #[test]
    fn pop_max_segment() {
        let mut queue = BucketQueue::<Segment<&str, i8>>::new();
        queue.push("first", 0, 1);
        queue.push("third", 2, 1);
        queue.push("second", 0, 2);
        queue.push("fourth", 2, 3);

        assert_eq!(queue.pop_max_segment_min(), Some(("third", 1)));
        assert_eq!(queue.pop_max_segment_min(), Some(("fourth", 3)));
        assert_eq!(queue.pop_max_segment_min(), Some(("first", 1)));
    }
}
