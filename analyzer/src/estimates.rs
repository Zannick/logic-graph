use crate::context::*;
use crate::steiner::approx::ApproxSteiner;
use crate::steiner::graph::*;
use crate::steiner::*;
use crate::world::*;
use crate::CommonHasher;
use lru::LruCache;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

pub const UNREASONABLE_TIME: u32 = 1 << 30;

// What we basically need is a helper that contains the necessary cache elements
// for scoring, that the DB can fall back to. Probably better than bloating the
// DB struct and functionality.
pub struct ContextScorer<'w, W, S, LI, EI, A> {
    // the cache is a map from start point and remaining locations to u64
    // but we also hold the Algo which contains precalculations for generating
    world: &'w W,
    algo: A,

    known_costs: Mutex<LruCache<(S, Vec<LI>, Vec<Edge<EI>>), u64, CommonHasher>>,
    required_locations: Vec<LI>,

    estimates: AtomicUsize,
    cached_estimates: AtomicUsize,
}

impl<'w, W, A>
    ContextScorer<'w, W, <W::Exit as Exit>::SpotId, <W::Location as Location>::LocId, EdgeId<W>, A>
where
    W: World,
    A: SteinerAlgo<NodeId<W>, EdgeId<W>>,
{
    fn new<T>(world: &'w W, startctx: &T, cache_size: usize) -> Self
    where
        T: Ctx<World = W>,
    {
        let required_locations: Vec<_> = world
            .required_items()
            .into_iter()
            .flat_map(|(item, _)| world.get_item_locations(item))
            .collect();
        Self {
            world,
            algo: A::from_graph(build_simple_graph(world, startctx)),
            known_costs: Mutex::new(LruCache::with_hasher(
                NonZeroUsize::new(cache_size).unwrap(),
                CommonHasher::default(),
            )),
            required_locations,
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
        W::Location: Location<Context = T>,
    {
        if self.world.won(ctx) {
            return 0;
        }
        // items_needed gives us the remaining items, while get_item_locations gives us all locations,
        // even ones already visited
        let item_sets: Vec<_> = self
            .world
            .items_needed(ctx)
            .into_iter()
            .map(|(item, ct)| (self.world.get_item_locations(item), ct))
            .collect();
        let subsets: Vec<_> = item_sets
            .iter()
            .filter_map(|(ilist, ct)| {
                if ilist.len() > (*ct).try_into().unwrap() {
                    Some((
                        // technically we should reduce this to todo locations but it doesn't matter
                        ilist.iter().copied().collect::<HashSet<_, CommonHasher>>(),
                        *ct,
                    ))
                } else {
                    None
                }
            })
            .collect();
        self.estimate_time_to_get(
            ctx,
            item_sets
                .into_iter()
                .flat_map(|(v, _)| v)
                .filter(|&loc_id| !ctx.visited(loc_id))
                .collect(),
            subsets,
        )
    }

    pub fn required_visits<T>(&self, ctx: &T) -> usize
    where
        T: Ctx<World = W>,
    {
        self.required_locations
            .iter()
            .filter(|&loc_id| ctx.visited(*loc_id))
            .count()
    }

    /// Number of objective locations left unvisited.
    /// aka the maximum additional progress count from this state.
    pub fn remaining_visits<T>(&self, ctx: &T) -> usize
    where
        T: Ctx<World = W>,
    {
        self.required_locations
            .iter()
            .filter(|&loc_id| !ctx.visited(*loc_id))
            .count()
    }

    pub fn remaining_locations<T>(&self, ctx: &T) -> Vec<<W::Location as Location>::LocId>
    where
        T: Ctx<World = W>,
    {
        self.required_locations
            .iter()
            .filter(|&loc_id| !ctx.visited(*loc_id))
            .copied()
            .collect()
    }

    /// Returns the estimate amount of time to get the specified locations from
    /// the current state. Does not check whether these locations are todo.
    pub fn estimate_time_to_get<T>(
        &self,
        ctx: &T,
        required: Vec<<W::Location as Location>::LocId>,
        subsets: Vec<(HashSet<<W::Location as Location>::LocId, CommonHasher>, i16)>,
    ) -> u64
    where
        T: Ctx<World = W>,
        W::Location: Location<Context = T>,
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
                if wp.can_access(ctx, self.world) {
                    Some(self.algo.graph().new_edge(
                        ExternalEdgeId::Warp(wp.id()),
                        ExternalNodeId::Spot(ctx.position()),
                        ExternalNodeId::Spot(Warp::dest(wp, ctx, self.world)),
                        wp.base_time().try_into().unwrap(),
                    ))
                } else {
                    None
                }
            })
            .chain(self.world.get_global_actions().iter().filter_map(|act| {
                if Action::dest(act, ctx, self.world) != Default::default()
                    && act.can_access(ctx, self.world)
                {
                    Some(self.algo.graph().new_edge(
                        ExternalEdgeId::Action(act.id()),
                        ExternalNodeId::Spot(ctx.position()),
                        ExternalNodeId::Spot(Action::dest(act, ctx, self.world)),
                        act.base_time().try_into().unwrap(),
                    ))
                } else {
                    None
                }
            }))
            .collect();

        let key: (_, Vec<_>, Vec<_>) = (pos, required, extra_edges);
        let mut locked_map = self.known_costs.lock().unwrap();
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
            let node_subsets = subsets.iter().map(|(set, ct)| {
                (
                    set.iter()
                        .map(|loc_id| loc_to_graph_node(self.world, *loc_id))
                        .collect::<HashSet<_, CommonHasher>>(),
                    *ct,
                )
            });
            let c = if let Some(ApproxSteiner { arborescence, cost }) = self.algo.compute(
                spot_to_graph_node::<W>(ctx.position()),
                nodes.collect(),
                node_subsets.collect(),
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
                UNREASONABLE_TIME.into()
            };
            {
                let mut locked_map = self.known_costs.lock().unwrap();
                locked_map.push(key, c);
            }
            self.estimates.fetch_add(1, Ordering::Release);
            c
        }
    }
}

impl<'w, W>
    ContextScorer<
        'w,
        W,
        <W::Exit as Exit>::SpotId,
        <W::Location as Location>::LocId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    >
where
    W: World,
{
    pub fn shortest_paths<T>(world: &'w W, startctx: &T, cache_size: usize) -> Self
    where
        T: Ctx<World = W>,
    {
        let now = Instant::now();
        let sp = Self::new(world, startctx, cache_size);
        log::info!("Built shortest_paths scorer in {:?}", now.elapsed());
        sp
    }

    pub fn shortest_paths_tree_only<T>(
        world: &'w W,
        startctx: &T,
    ) -> ShortestPaths<NodeId<W>, EdgeId<W>>
    where
        T: Ctx<World = W>,
    {
        let now = Instant::now();
        let sp = ShortestPaths::from_graph(build_simple_graph(world, startctx));
        log::info!("Built shortest_paths tree only in {:?}", now.elapsed());
        sp
    }

    pub fn get_algo(&self) -> &ShortestPaths<NodeId<W>, EdgeId<W>> {
        &self.algo
    }
}
