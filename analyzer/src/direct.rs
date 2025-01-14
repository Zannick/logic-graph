use crate::context::{history_to_full_time_series, Ctx, HistoryAlias};
use crate::matchertrie::MatcherTrie;
use crate::observer::{Observer, TrieMatcher};
use crate::route::{PartialRoute, RouteStep};
use crate::steiner::graph::ExternalNodeId;
use crate::steiner::{EdgeId, NodeId, ShortestPaths};
use crate::CommonHasher;
use crate::{new_hashmap, world::*};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

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
    pub hits: AtomicUsize,
    pub min_hits: AtomicUsize,
    pub improves: AtomicUsize,
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
            hits: 0.into(),
            min_hits: 0.into(),
            improves: 0.into(),
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
        { self.map.lock().unwrap().get(&dest)?.clone() }
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

    pub fn totals(&self) -> (usize, usize) {
        let tries: Vec<_> = self.map.lock().unwrap().values().cloned().collect();
        (
            tries.iter().map(|trie| trie.size()).sum(),
            tries.iter().map(|trie| trie.num_values()).sum(),
        )
    }
}
