//! Uses bucket_queue to create a queue ordered by one priority measure
//! but segmented by another priority measure, such that callers can
//! retrieve the minimum (or maximum) of the SegmentedBucketQueue overall or the
//! minimum/maximum of a given segment

use crate::CommonHasher;
use bucket_queue::{Bucket, BucketQueue, Queue};
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
    fn push(&mut self, item: Self::Item, priority: P);
    fn pop_min(&mut self) -> Option<(Self::Item, P)>;
    fn pop_max(&mut self) -> Option<(Self::Item, P)>;

    fn peek_min(&self) -> Option<(&Self::Item, &P)>;
    fn peek_max(&self) -> Option<(&Self::Item, &P)>;
    fn min_priority(&self) -> Option<&P>;
    fn max_priority(&self) -> Option<&P>;

    fn iter(&self) -> priority_queue::core_iterators::Iter<Self::Item, P>
    where
        Self::Item: Hash + Eq;
}

impl<I, P> SegmentBucket<P> for Segment<I, P>
where
    I: Hash + Eq,
    P: Ord,
{
    fn push(&mut self, item: I, priority: P) {
        self.pq.push(item, priority);
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

    fn iter(&self) -> priority_queue::core_iterators::Iter<I, P>
    where
        Self::Item: Hash + Eq,
    {
        self.pq.iter()
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
    type Item = (&'b B::Item, &'b P);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(x) = self.iter.next() {
                return Some(x);
            }
            if self.bucket < self.max {
                self.bucket += 1;
                self.iter = self.q.bucket_for_peeking(self.bucket).unwrap().iter();
            } else {
                return None;
            }
        }
    }
}

pub trait SegmentedBucketQueue<'b, B: SegmentBucket<P> + 'b, P: Ord>: Queue<B> {
    fn push(&mut self, item: B::Item, segment: usize, priority: P) {
        self.bucket_for_adding(segment).push(item, priority);
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
            .min_by_key(|&s| self.bucket_for_peeking(s).unwrap().min_priority())?;
        self.bucket_for_removing(segment)?.pop_min()
    }
    fn pop_max(&mut self) -> Option<(B::Item, P)> {
        let segment = (self.min_priority()?..=self.max_priority()?)
            .max_by_key(|&s| self.bucket_for_peeking(s).unwrap().max_priority())?;
        self.bucket_for_removing(segment)?.pop_max()
    }

    fn peek_min(&'b mut self) -> Option<(&'b B::Item, &'b P)> {
        let segment = (self.min_priority()?..=self.max_priority()?)
            .min_by_key(|&s| self.bucket_for_peeking(s).unwrap().min_priority())?;
        self.bucket_for_peeking(segment)?.peek_min()
    }
    fn peek_max(&'b mut self) -> Option<(&'b B::Item, &'b P)> {
        let segment = (self.min_priority()?..=self.max_priority()?)
            .max_by_key(|&s| self.bucket_for_peeking(s).unwrap().max_priority())?;
        self.bucket_for_peeking(segment)?.peek_max()
    }

    fn pop_min_segment_min(&'b mut self, min_segment: usize) -> Option<(B::Item, P)> {
        let min_segment = std::cmp::min(min_segment, self.min_priority()?);
        for segment in min_segment..=self.max_priority()? {
            let x = if let Some(b) = self.bucket_for_replacing(segment) {
                b.pop_min()
            } else {
                None
            };
            if x.is_some() {
                // Update the index
                self.items_replaced(segment, 1, 0);
                return x;
            }
        }
        None
    }

    fn pop_max_segment_min(&'b mut self) -> Option<(B::Item, P)> {
        let segment = self.max_priority()?;
        self.bucket_for_removing(segment)?.pop_min()
    }

    fn bucket_sizes(&self) -> Vec<usize> {
        if let Some(max) = self.max_priority() {
            (0..=max)
                .into_iter()
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

    fn iter(&'b self) -> Iter<'b, Self, B, P>
    where
        B::Item: Hash + Eq,
        Self: Sized,
    {
        let min = self.min_priority().unwrap_or_default();
        let max = self.max_priority().unwrap_or_default();
        Iter {
            q: &self,
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

        assert_eq!(queue.pop_min_segment_min(1), Some(("third", 1)));
        assert_eq!(queue.pop_min_segment_min(2), Some(("fourth", 3)));
        assert_eq!(queue.pop_min_segment_min(1), None);
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
