use crate::context::*;
use crate::steiner::graph::*;
use crate::steiner::*;
use crate::world::*;
use crate::{new_hashmap, CommonHasher};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

// What we basically need is a helper that contains the necessary cache elements
// for scoring, that the DB can fall back to. Probably better than bloating the
// DB struct and functionality.
pub struct ContextScorer<'w, W, S, LI, A> {
    // the cache is a map from start point and remaining locations to u64
    // but we also hold the Algo which contains precalculations for generating
    world: &'w W,
    algo: A,

    known_costs: Mutex<HashMap<(S, Vec<LI>), u64, CommonHasher>>,

    estimates: AtomicUsize,
    cached_estimates: AtomicUsize,
}

impl<'w, W, S, L, E, A> ContextScorer<'w, W, S, L::LocId, A>
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
        let mut pos = ctx.last();
        if pos == S::default() {
            pos = ctx.position();
        }
        let key: (S, Vec<_>) = (
            pos,
            self.world
                .items_needed(ctx)
                .into_iter()
                .map(|(item, _)| self.world.get_item_locations(item))
                .flatten()
                .collect(),
        );
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
            let c = if let Some(c) = self.algo.compute_cost(
                spot_to_graph_node::<W, E>(ctx.position()),
                nodes.collect(),
                extra_edges,
            ) {
                c
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

impl<'w, W, S, L, E> ContextScorer<'w, W, S, L::LocId, ShortestPaths<NodeId<W>, EdgeId<W>>>
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
