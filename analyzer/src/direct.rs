use crate::context::{history_to_full_time_series, Ctx, HistoryAlias};
use crate::matchertrie::MatcherTrie;
use crate::observer::{Observer, TrieMatcher};
use crate::steiner::graph::ExternalNodeId;
use crate::steiner::{EdgeId, NodeId, ShortestPaths};
use crate::CommonHasher;
use crate::{new_hashmap, world::*};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

// A route is very much like a solution, but we want to track all the step times
// and cache them together so we can keep just the smallest.
// TODO: Maybe we should do this for solutions as well?

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RouteStep<T: Ctx> {
    pub step: HistoryAlias<T>,
    pub time: u32,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PartialRoute<T: Ctx> {
    pub route: Arc<Vec<RouteStep<T>>>,
    pub start: usize,
    pub end: usize,
    pub time: u32,
}

impl<T: Ctx> PartialOrd for PartialRoute<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl<T: Ctx> Ord for PartialRoute<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl<T: Ctx> PartialRoute<T> {
    pub fn new(route: Arc<Vec<RouteStep<T>>>, start: usize, end: usize) -> Self {
        let time = route[start..end].iter().map(|rs| rs.time).sum();
        Self {
            route,
            start,
            end,
            time,
        }
    }
}

pub struct DirectPaths<W, T, TM>
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
}

// Given a route A -> B -> ... -> X
// we can create partial routes for every separate start step along the way.
// Each start step will have an observation set

impl<W, T, TM> DirectPaths<W, T, TM>
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
        }
    }

    pub fn min_free_time_to(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        start: <W::Exit as Exit>::SpotId,
    ) -> Option<u32> {
        self.free_sp
            .min_distance(ExternalNodeId::Spot(start), ExternalNodeId::Spot(dest))
            .map(|u| u as u32)
    }

    pub fn shortest_known_route_to(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        ctx: &T,
    ) -> Option<PartialRoute<T>> {
        // Clone the RC and avoid holding the map lock
        { self.map.lock().unwrap()[&dest].clone() }
            .lookup(ctx)
            .into_iter()
            .min_by_key(|pr| pr.time)
    }

    pub fn insert_route(
        &self,
        dest: <W::Exit as Exit>::SpotId,
        startctx: &T,
        world: &W,
        history: &[HistoryAlias<T>],
    ) {
        let mut map = self.map.lock().unwrap();
        if !map.contains_key(&dest) {
            map.insert(dest, Arc::new(MatcherTrie::default()));
        }

        let trie = map[&dest].clone();
        drop(map);

        let (full_series, _) =
            history_to_full_time_series(startctx, world, history.iter().copied());
        let route: Arc<Vec<_>> = Arc::new(
            full_series
                .iter()
                .map(|&(_, step, time)| RouteStep { step, time })
                .collect(),
        );

        // For solutions, the initial observation is the victory condition.
        // For partial routes, we don't need any initial observation at all.
        let mut solve = <T::Observer as Default>::default();

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
}
