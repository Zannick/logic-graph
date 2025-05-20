use crate::context::{history_to_full_time_series, history_to_partial_route, Ctx, HistoryAlias};
use crate::db::RouteDb;
use crate::matchertrie::MatcherTrie;
use crate::observer::{Observer, TrieMatcher};
use crate::route::{PartialRoute, RouteStep};
use crate::steiner::graph::ExternalNodeId;
use crate::steiner::{EdgeId, NodeId, ShortestPaths};
use crate::CommonHasher;
use crate::{new_hashmap, world::*};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub trait DirectPaths<W, T>
where
    W: World,
    T: Ctx<World = W> + Debug,
    W::Location: Location<Context = T>,
{
    fn min_free_time_to(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        start: <W::Exit as Exit>::SpotId,
    ) -> Option<u32>;

    fn shortest_known_route_to(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        ctx: &T,
    ) -> Option<PartialRoute<T>>;

    fn insert_route(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        startctx: &T,
        world: &W,
        history: &[HistoryAlias<T>],
    );

    // Counters
    fn count_hit(&self);
    fn count_min_hit(&self);
    fn count_improvement(&self);
    fn count_fail(&self);
    fn count_expire(&self);
    fn count_dead_end(&self);
}

pub struct DirectPathsMap<W, T, TM>
where
    W: World,
    T: Ctx<World = W> + Debug,
    W::Location: Location<Context = T>,
    TM: TrieMatcher<PartialRoute<T>, Struct = T>,
{
    map: Mutex<
        HashMap<<W::Exit as Exit>::SpotId, Arc<MatcherTrie<TM, PartialRoute<T>>>, CommonHasher>,
    >,
    free_sp: ShortestPaths<NodeId<W>, EdgeId<W>>,
    pub hits: AtomicUsize,
    pub min_hits: AtomicUsize,
    pub improves: AtomicUsize,

    pub fails: AtomicUsize,
    pub expires: AtomicUsize,
    pub deadends: AtomicUsize,
}

// Given a route A -> B -> ... -> X
// we can create partial routes for every separate start step along the way.
// Each start step will have an observation set

impl<W, T, TM> DirectPathsMap<W, T, TM>
where
    W: World,
    T: Ctx<World = W> + Debug,
    W::Location: Location<Context = T>,
    TM: TrieMatcher<PartialRoute<T>, Struct = T>,
{
    pub fn new(free_sp: ShortestPaths<NodeId<W>, EdgeId<W>>) -> Self {
        Self {
            map: Mutex::new(new_hashmap()),
            free_sp,
            hits: 0.into(),
            min_hits: 0.into(),
            improves: 0.into(),
            fails: 0.into(),
            expires: 0.into(),
            deadends: 0.into(),
        }
    }

    pub fn totals(&self) -> (usize, usize) {
        let tries: Vec<_> = self.map.lock().unwrap().values().cloned().collect();
        (
            tries.iter().map(|trie| trie.size()).sum(),
            tries.iter().map(|trie| trie.num_values()).sum(),
        )
    }
}

impl<W, T, TM> DirectPaths<W, T> for DirectPathsMap<W, T, TM>
where
    W: World,
    T: Ctx<World = W> + Debug,
    W::Location: Location<Context = T>,
    TM: TrieMatcher<PartialRoute<T>, Struct = T>,
{
    fn min_free_time_to(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        start: <W::Exit as Exit>::SpotId,
    ) -> Option<u32> {
        self.free_sp
            .min_distance(ExternalNodeId::Spot(start), ExternalNodeId::Spot(dest))
            .map(|u| u as u32)
    }

    fn shortest_known_route_to(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        ctx: &T,
    ) -> Option<PartialRoute<T>> {
        // Clone the RC and avoid holding the map lock
        { self.map.lock().unwrap().get(&dest)?.clone() }
            .lookup(ctx)
            .into_iter()
            .min_by_key(|pr| pr.time)
    }

    fn insert_route(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        startctx: &T,
        world: &W,
        history: &[HistoryAlias<T>],
    ) {
        // Lock the map while retrieving the trie pointer
        let trie = {
            let mut map = self.map.lock().unwrap();
            if !map.contains_key(&dest) {
                map.insert(dest, Arc::new(MatcherTrie::default()));
            }

            map[&dest].clone()
        };

        let (full_series, _) =
            history_to_full_time_series(startctx, world, history.iter().copied());
        let route: Arc<Vec<_>> = Arc::new(
            full_series
                .iter()
                .map(|&(_, step, time)| RouteStep::<T> { step, time })
                .collect(),
        );

        // For solutions, the initial observation is the victory condition.
        // For partial routes, we don't need any initial observation at all (except position for the root).
        let mut solve = <T::Observer as Default>::default();
        solve.observe_position();

        let mut total_time = 0;
        for (idx, (state, step, time)) in full_series.iter().enumerate().rev() {
            // Basic process of iterating backwards:
            // 1. Observe the history step requirements/effects itself.
            state.observe_replay(world, *step, &mut solve);

            // 2. Apply the observations in reverse order.
            solve.apply_observations();

            // 3. Insert the new observation list.
            total_time += time;
            trie.insert(
                solve.to_vec(state),
                PartialRoute {
                    route: route.clone(),
                    start: idx,
                    end: route.len(),
                    time: total_time,
                },
            );
        }
    }

    fn count_hit(&self) {
        self.hits.fetch_add(1, Ordering::Release);
    }
    fn count_min_hit(&self) {
        self.min_hits.fetch_add(1, Ordering::Release);
    }
    fn count_improvement(&self) {
        self.improves.fetch_add(1, Ordering::Release);
    }
    fn count_fail(&self) {
        self.fails.fetch_add(1, Ordering::Release);
    }
    fn count_expire(&self) {
        self.expires.fetch_add(1, Ordering::Release);
    }
    fn count_dead_end(&self) {
        self.deadends.fetch_add(1, Ordering::Release);
    }
}

pub struct DirectPathsDb<W, T>
where
    W: World,
    T: Ctx<World = W> + Debug,
    W::Location: Location<Context = T>,
{
    rdb: RouteDb<T>,
    free_sp: ShortestPaths<NodeId<W>, EdgeId<W>>,
    pub hits: AtomicUsize,
    pub min_hits: AtomicUsize,
    pub improves: AtomicUsize,

    pub fails: AtomicUsize,
    pub expires: AtomicUsize,
    pub deadends: AtomicUsize,
}

impl<W, T> DirectPathsDb<W, T>
where
    W: World,
    T: Ctx<World = W> + Debug,
    W::Location: Location<Context = T>,
{
    pub fn new(free_sp: ShortestPaths<NodeId<W>, EdgeId<W>>, rdb: RouteDb<T>) -> Self {
        Self {
            rdb,
            free_sp,
            hits: 0.into(),
            min_hits: 0.into(),
            improves: 0.into(),
            fails: 0.into(),
            expires: 0.into(),
            deadends: 0.into(),
        }
    }

    pub fn totals(&self) -> (usize, usize) {
        (self.rdb.num_routes(), self.rdb.trie_size())
    }
}

impl<W, T> DirectPaths<W, T> for DirectPathsDb<W, T>
where
    W: World,
    T: Ctx<World = W> + Debug,
    W::Location: Location<Context = T>,
{
    fn min_free_time_to(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        start: <W::Exit as Exit>::SpotId,
    ) -> Option<u32> {
        self.free_sp
            .min_distance(ExternalNodeId::Spot(start), ExternalNodeId::Spot(dest))
            .map(|u| u as u32)
    }

    fn shortest_known_route_to(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        ctx: &T,
    ) -> Option<PartialRoute<T>> {
        self.rdb
            .best_known_route(ctx, dest)
            .unwrap()
            .map(PartialRoute::from)
    }

    fn insert_route(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        startctx: &T,
        world: &W,
        history: &[HistoryAlias<T>],
    ) {
        let route = history_to_partial_route(startctx, world, history.iter().copied());

        self.rdb.insert_route(startctx, world, dest, &route);
    }

    fn count_hit(&self) {
        self.hits.fetch_add(1, Ordering::Release);
    }
    fn count_min_hit(&self) {
        self.min_hits.fetch_add(1, Ordering::Release);
    }
    fn count_improvement(&self) {
        self.improves.fetch_add(1, Ordering::Release);
    }
    fn count_fail(&self) {
        self.fails.fetch_add(1, Ordering::Release);
    }
    fn count_expire(&self) {
        self.expires.fetch_add(1, Ordering::Release);
    }
    fn count_dead_end(&self) {
        self.deadends.fetch_add(1, Ordering::Release);
    }
}
