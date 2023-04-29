use crate::context::*;
use crate::steiner::approx::ApproxSteiner;
use crate::steiner::graph::*;
use crate::steiner::*;
use crate::world::*;
use crate::{new_hashmap, CommonHasher};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

// What we basically need is a helper that contains the necessary cache elements
// for scoring, that the DB can fall back to. Probably better than bloating the
// DB struct and functionality.
pub struct ContextScorer<'w, W, S, LI, EI, A> {
    // the cache is a map from start point and remaining locations to u64
    // but we also hold the Algo which contains precalculations for generating
    world: &'w W,
    algo: A,

    known_costs: Mutex<HashMap<(S, Vec<LI>, Vec<Edge<EI>>), u64, CommonHasher>>,

    estimates: AtomicUsize,
    cached_estimates: AtomicUsize,
}

impl<'w, W, S, L, E, A> ContextScorer<'w, W, S, L::LocId, EdgeId<W>, A>
where
    W: World<Location = L, Exit = E>,
    L: Location<ExitId = E::ExitId>,
    E: Exit<SpotId = S>,
    A: SteinerAlgo<NodeId<W>, EdgeId<W>>,
    S: Id + Default,
{
    fn new<T>(world: &'w W, startctx: &T) -> Self
    where
        T: Ctx<World = W>,
    {
        Self {
            world,
            algo: A::from_graph(build_simple_graph(world, startctx)),
            known_costs: Mutex::new(new_hashmap()),
            estimates: 0.into(),
            cached_estimates: 0.into(),
        }
    }

    pub fn estimates(&self) -> usize {
        self.estimates.load(Ordering::Acquire)
    }

    pub fn cached_estimates(&self) -> usize {
        self.cached_estimates.load(Ordering::Acquire)
    }

    pub fn estimate_remaining_time<T>(&self, ctx: &T) -> u64
    where
        T: Ctx<World = W>,
        L: Location<Context = T>,
    {
        if self.world.won(ctx) {
            return 0;
        }
        self.estimate_time_to_get(
            ctx,
            self.world
                .items_needed(ctx)
                .into_iter()
                .map(|(item, _)| self.world.get_item_locations(item))
                .flatten()
                .filter(|&loc_id| ctx.todo(loc_id))
                .collect(),
        )
    }

    pub fn required_visits<T>(&self, ctx: &T) -> usize
    where
        T: Ctx<World = W>,
        L: Location<Context = T>,
    {
        self.world
            .items_needed(ctx)
            .into_iter()
            .map(|(item, _)| self.world.get_item_locations(item))
            .flatten()
            .filter(|&loc_id| ctx.visited(loc_id))
            .count()
    }

    /// Returns the estimate amount of time to get the specified locations from
    /// the current state. Does not check whether these locations are todo.
    pub fn estimate_time_to_get<T>(&self, ctx: &T, required: Vec<<L as Location>::LocId>) -> u64
    where
        T: Ctx<World = W>,
        L: Location<Context = T>,
    {
        if required.is_empty() || self.world.won(ctx) {
            return 0;
        }
        let pos = ctx.position();
        let extra_edges: Vec<_> = self
            .world
            .get_warps()
            .iter()
            .filter_map(|wp| {
                if wp.can_access(ctx) {
                    Some(self.algo.graph().new_edge(
                        ExternalEdgeId::Warp(wp.id()),
                        ExternalNodeId::Spot(ctx.position()),
                        ExternalNodeId::Spot(wp.dest(ctx)),
                        wp.time().try_into().unwrap(),
                    ))
                } else {
                    None
                }
            })
            .chain(self.world.get_global_actions().iter().filter_map(|act| {
                if act.dest(ctx) != Default::default() && act.can_access(ctx) {
                    Some(self.algo.graph().new_edge(
                        ExternalEdgeId::Action(act.id()),
                        ExternalNodeId::Spot(ctx.position()),
                        ExternalNodeId::Spot(act.dest(ctx)),
                        act.time().try_into().unwrap(),
                    ))
                } else {
                    None
                }
            }))
            .collect();

        let key: (S, Vec<_>, Vec<_>) = (pos, required, extra_edges);
        let locked_map = self.known_costs.lock().unwrap();
        if let Some(&c) = locked_map.get(&key) {
            drop(locked_map);
            self.cached_estimates.fetch_add(1, Ordering::Release);
            c
        } else {
            drop(locked_map);
            let nodes = key
                .1
                .iter()
                .map(|loc_id| loc_to_graph_node(self.world, *loc_id));
            let c = if let Some(ApproxSteiner { arborescence, cost }) = self.algo.compute(
                spot_to_graph_node::<W, E>(ctx.position()),
                nodes.collect(),
                &key.2,
            ) {
                // Extra warp cost is number of "branches" times min_warp_time
                // Number of branches is number of edges minus number of unique starting nodes
                // Only count the spot to spot edges
                let mut edges = 0;
                let unique_nodes: HashSet<_> = arborescence
                    .into_iter()
                    .filter_map(|e| match e {
                        ExternalEdgeId::Spots(src, _) => {
                            edges += 1;
                            Some(src)
                        }
                        _ => None,
                    })
                    .collect();
                let min_warp_time: u64 = self.world.min_warp_time().into();
                let warp_cost = min_warp_time
                    * <usize as TryInto<u64>>::try_into(edges - unique_nodes.len()).unwrap();
                cost + warp_cost
            } else {
                // A sufficiently large number.
                1 << 30
            };
            {
                let mut locked_map = self.known_costs.lock().unwrap();
                locked_map.insert(key, c);
            }
            self.estimates.fetch_add(1, Ordering::Release);
            c
        }
    }
}

impl<'w, W, S, L, E>
    ContextScorer<'w, W, S, L::LocId, EdgeId<W>, ShortestPaths<NodeId<W>, EdgeId<W>>>
where
    W: World<Location = L, Exit = E>,
    L: Location<ExitId = E::ExitId>,
    E: Exit<SpotId = S>,
    S: Id + Default,
{
    pub fn shortest_paths<T>(world: &'w W, startctx: &T) -> Self
    where
        T: Ctx<World = W>,
    {
        Self::new(world, startctx)
    }
}
