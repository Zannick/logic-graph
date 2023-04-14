use crate::context::*;
use crate::steiner::graph::*;
use crate::steiner::sp::ShortestPaths;
use crate::steiner::*;
use crate::world::*;
use crate::{new_hashmap, CommonHasher};
use std::collections::HashMap;

// What we basically need is a helper that contains the necessary cache elements
// for scoring, that the DB can fall back to. Probably better than bloating the
// DB struct and functionality.
pub struct ContextScorer<'w, W, S, LI, A> {
    // the cache is a map from start point and remaining locations to u64
    // but we also hold the Algo which contains precalculations for generating
    world: &'w W,
    algo: A,

    known_costs: HashMap<(S, Vec<LI>), u64, CommonHasher>,
}

impl<'w, W, S, L, E, A> ContextScorer<'w, W, S, L::LocId, A>
where
    W: World<Location = L, Exit = E>,
    L: Location<ExitId = E::ExitId>,
    E: Exit<SpotId = S>,
    A: SteinerAlgo<NodeId<W>, EdgeId<W>>,
    S: Id,
{
    fn new<T>(world: &'w W, startctx: &T) -> Self
    where
        T: Ctx<World = W>,
    {
        Self {
            world,
            algo: A::from_graph(build_simple_graph(world, startctx)),
            known_costs: new_hashmap(),
        }
    }

    pub fn estimate_remaining_time<T>(&mut self, ctx: &T) -> u64
    where
        T: Ctx<World = W>,
        L: Location<Context = T>,
    {
        let key: (S, Vec<_>) = (
            ctx.position(),
            self.world
                .items_needed(ctx)
                .into_iter()
                .map(|(item, _)| self.world.get_item_locations(item))
                .flatten()
                .collect(),
        );
        if let Some(c) = self.known_costs.get(&key) {
            *c
        } else {
            let nodes = key
                .1
                .iter()
                .map(|loc_id| loc_to_graph_node(self.world, *loc_id));
            let c = self
                .algo
                .compute_cost(spot_to_graph_node::<W, E>(ctx.position()), nodes.collect())
                .unwrap();
            self.known_costs.insert(key, c);
            c
        }
    }
}

impl<'w, W, S, L, E> ContextScorer<'w, W, S, L::LocId, ShortestPaths<NodeId<W>, EdgeId<W>>>
where
    W: World<Location = L, Exit = E>,
    L: Location<ExitId = E::ExitId>,
    E: Exit<SpotId = S>,
    S: Id,
{
    pub fn shortest_paths<T>(world: &'w W, startctx: &T) -> Self
    where
        T: Ctx<World = W>,
    {
        Self::new(world, startctx)
    }
}
