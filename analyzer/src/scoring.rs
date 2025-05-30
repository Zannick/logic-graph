use crate::context::*;
use crate::estimates::ContextScorer;
use crate::steiner::*;
use crate::world::*;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU32, Ordering};

pub(crate) type TimeSinceScore = (u32, u32);

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct BestTimes {
    pub elapsed: u32,
    pub time_since_visit: u32,
    pub estimated_remaining: u32,
}

pub trait MetricKey {
    /// Returns the first sort field of the score.
    fn get_score_primary_from_heap_key(key: &[u8]) -> u32;
    fn get_total_estimate_from_heap_key(key: &[u8]) -> u32;
}

pub trait EstimatorWrapper<'w, W: World + 'w> {
    fn estimator(
        &self,
    ) -> &ContextScorer<
        'w,
        W,
        <W::Exit as Exit>::SpotId,
        <W::Location as Location>::LocId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    >;

    /// Estimates the remaining time to the goal.
    fn estimated_remaining_time<T>(&self, ctx: &T) -> u32
    where
        T: Ctx<World = W>,
        W::Location: Location<Context = T>,
    {
        self.estimator()
            .estimate_remaining_time(ctx)
            .try_into()
            .unwrap()
    }

    /// Returns the number of unique states we've estimated remaining time for.
    /// Winning states aren't counted in this.
    fn estimates(&self) -> usize {
        self.estimator().estimates()
    }

    /// Returns the number of cache hits for estimated remaining time.
    /// Winning states aren't counted in this.
    fn cached_estimates(&self) -> usize {
        self.estimator().cached_estimates()
    }
}

pub trait ScoreMetric<'w, W: World + 'w, T: Ctx, const KEY_SIZE: usize>:
    MetricKey + EstimatorWrapper<'w, W> + Sized
{
    type Score: Copy + Debug + Ord;

    fn new(world: &'w W, startctx: &T) -> Self;
    fn score_from_times(&self, best_times: BestTimes) -> Self::Score;
    fn score_from_heap_key(&self, key: &[u8]) -> Self::Score;
    fn get_heap_key(&self, el: &T, score: Self::Score) -> [u8; KEY_SIZE];
    fn new_heap_key(&self, old_key: &[u8], new_score: Self::Score) -> [u8; KEY_SIZE];

    // Using &self to avoid trying to provide the metric type in the heap's DbType alias
    fn total_estimate_from_score(&self, score: Self::Score) -> u32;
    fn score_primary(&self, score: Self::Score) -> u32;
}

pub struct TimeSinceAndElapsed<'w, W: World> {
    seq: AtomicU32,
    estimator: ContextScorer<
        'w,
        W,
        <W::Exit as Exit>::SpotId,
        <W::Location as Location>::LocId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    >,
}

impl<'w, W> EstimatorWrapper<'w, W> for TimeSinceAndElapsed<'w, W>
where
    W: World + 'w,
{
    fn estimator(
        &self,
    ) -> &ContextScorer<
        'w,
        W,
        <W::Exit as Exit>::SpotId,
        <W::Location as Location>::LocId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    > {
        &self.estimator
    }
}

impl<'w, W: World> MetricKey for TimeSinceAndElapsed<'w, W> {    
    fn get_score_primary_from_heap_key(key: &[u8]) -> u32 {
        u32::from_be_bytes(key[4..8].try_into().unwrap())
    }
    fn get_total_estimate_from_heap_key(key: &[u8]) -> u32 {
        u32::from_be_bytes(key[8..12].try_into().unwrap())
    }
}

impl<'w, W, T, L, E> ScoreMetric<'w, W, T, 16> for TimeSinceAndElapsed<'w, W>
where
    W: World<Location = L, Exit = E> + 'w,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    E: Exit<Context = T>,
{
    type Score = TimeSinceScore;

    fn new(world: &'w W, startctx: &T) -> Self {
        Self {
            seq: 0.into(),
            estimator: ContextScorer::shortest_paths(world, startctx, 32_768),
        }
    }

    // TODO: make a type alias or struct for best times
    fn score_from_times(
        &self,
        BestTimes {
            elapsed,
            time_since_visit: time_since,
            estimated_remaining,
        }: BestTimes,
    ) -> TimeSinceScore {
        (time_since, elapsed + estimated_remaining)
    }

    fn score_from_heap_key(&self, key: &[u8]) -> TimeSinceScore {
        (
            u32::from_be_bytes(key[4..8].try_into().unwrap()),
            u32::from_be_bytes(key[8..12].try_into().unwrap()),
        )
    }

    fn get_heap_key(&self, el: &T, score: TimeSinceScore) -> [u8; 16] {
        let mut key: [u8; 16] = [0; 16];
        let progress: u32 = el.count_visits() as u32;
        key[0..4].copy_from_slice(&progress.to_be_bytes());
        key[4..8].copy_from_slice(&score.0.to_be_bytes());
        key[8..12].copy_from_slice(&score.1.to_be_bytes());
        key[12..16].copy_from_slice(&self.seq.fetch_add(1, Ordering::AcqRel).to_be_bytes());
        key
    }
    fn new_heap_key(&self, old_key: &[u8], new_score: TimeSinceScore) -> [u8; 16] {
        let mut key: [u8; 16] = [0; 16];
        key[0..4].copy_from_slice(&old_key[0..4]);
        key[4..8].copy_from_slice(&new_score.0.to_be_bytes());
        key[8..12].copy_from_slice(&new_score.1.to_be_bytes());
        key[12..16].copy_from_slice(&self.seq.fetch_add(1, Ordering::AcqRel).to_be_bytes());
        key
    }

    fn total_estimate_from_score(&self, score: Self::Score) -> u32 {
        score.1
    }
    fn score_primary(&self, score: Self::Score) -> u32 {
        score.0
    }
}

type EstimatedTime = u32;
pub struct EstimatedTimeMetric<'w, W: World> {
    seq: AtomicU32,
    estimator: ContextScorer<
        'w,
        W,
        <W::Exit as Exit>::SpotId,
        <W::Location as Location>::LocId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    >,
}

impl<'w, W> EstimatorWrapper<'w, W> for EstimatedTimeMetric<'w, W>
where
    W: World + 'w,
{
    fn estimator(
        &self,
    ) -> &ContextScorer<
        'w,
        W,
        <W::Exit as Exit>::SpotId,
        <W::Location as Location>::LocId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    > {
        &self.estimator
    }
}

impl<'w, W: World> MetricKey for EstimatedTimeMetric<'w, W> {
    fn get_score_primary_from_heap_key(key: &[u8]) -> u32 {
        u32::from_be_bytes(key[4..8].try_into().unwrap())
    }
    fn get_total_estimate_from_heap_key(key: &[u8]) -> u32 {
        u32::from_be_bytes(key[4..8].try_into().unwrap())
    }
}

impl<'w, W, T, L, E> ScoreMetric<'w, W, T, 12> for EstimatedTimeMetric<'w, W>
where
    W: World<Location = L, Exit = E> + 'w,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    E: Exit<Context = T>,
{
    type Score = EstimatedTime;

    fn new(world: &'w W, startctx: &T) -> Self {
        Self {
            seq: 0.into(),
            estimator: ContextScorer::shortest_paths(world, startctx, 32_768),
        }
    }

    fn score_from_times(
        &self,
        BestTimes {
            elapsed,
            estimated_remaining,
            ..
        }: BestTimes,
    ) -> EstimatedTime {
        elapsed + estimated_remaining
    }

    fn score_from_heap_key(&self, key: &[u8]) -> EstimatedTime {
        u32::from_be_bytes(key[4..8].try_into().unwrap())
    }

    fn get_heap_key(&self, el: &T, score: EstimatedTime) -> [u8; 12] {
        let mut key: [u8; 12] = [0; 12];
        let progress: u32 = el.count_visits() as u32;
        key[0..4].copy_from_slice(&progress.to_be_bytes());
        key[4..8].copy_from_slice(&score.to_be_bytes());
        key[8..12].copy_from_slice(&self.seq.fetch_add(1, Ordering::AcqRel).to_be_bytes());
        key
    }
    fn new_heap_key(&self, old_key: &[u8], new_score: EstimatedTime) -> [u8; 12] {
        let mut key: [u8; 12] = [0; 12];
        key[0..4].copy_from_slice(&old_key[0..4]);
        key[4..8].copy_from_slice(&new_score.to_be_bytes());
        key[8..12].copy_from_slice(&self.seq.fetch_add(1, Ordering::AcqRel).to_be_bytes());
        key
    }

    fn total_estimate_from_score(&self, score: Self::Score) -> u32 {
        score
    }
    fn score_primary(&self, score: Self::Score) -> u32 {
        score
    }
}
