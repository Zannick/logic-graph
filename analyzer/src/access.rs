//! Functions related to access graphs, and accessing locations.

use crate::a_star::*;
use crate::context::*;
use crate::greedy::greedy_search_from;
use crate::heap::HeapElement;
use crate::priority::LimitedPriorityQueue;
use crate::steiner::graph::ExternalNodeId;
use crate::steiner::{EdgeId, NodeId, ShortestPaths, SteinerAlgo};
use crate::world::*;
use crate::{new_hashmap, CommonHasher};
use ordered_float::OrderedFloat;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

static INITIAL_CAPACITY: usize = 1_024;
static MAX_STATES_FOR_SPOTS: usize = 16_384;
static MAX_STATES_FOR_LOCS: usize = 16_384;

/// Check whether there are available locations at this position.
pub fn spot_has_locations<W, T, L, E>(world: &W, ctx: &T) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    E: Exit<Context = T>,
{
    world
        .get_spot_locations(ctx.position())
        .iter()
        .any(|loc| ctx.todo(loc) && loc.can_access(ctx, world))
}

/// Check whether there are available actions at this position, including global actions.
pub fn spot_has_actions<W, T, L, E>(world: &W, ctx: &T) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    E: Exit<Context = T>,
{
    world
        .get_global_actions()
        .iter()
        .chain(world.get_spot_actions(ctx.position()))
        .any(|act| act.can_access(ctx, world))
}

fn expand<W, T, E, Wp>(
    world: &W,
    ctx: &ContextWrapper<T>,
    spot_map: &HashMap<E::SpotId, ContextWrapper<T>, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<HeapElement<T>>>,
    allow_local: bool,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    let movement_state = ctx.get().get_movement_state(world);
    let cedges = world.get_condensed_edges_from(ctx.get().position());
    if !cedges.is_empty() {
        for ce in cedges {
            if !spot_map.contains_key(&ce.dst) && ce.can_access(world, ctx.get(), movement_state) {
                let mut newctx = ctx.clone();
                newctx.move_condensed_edge(world, ce);
                let elapsed = newctx.elapsed();
                if elapsed <= max_time {
                    spot_heap.push(Reverse(HeapElement {
                        score: elapsed,
                        el: newctx,
                    }));
                }
            }
        }
        if allow_local {
            expand_local(world, ctx, movement_state, spot_map, max_time, spot_heap);
        }
    } else {
        expand_local(world, ctx, movement_state, spot_map, max_time, spot_heap);
    }

    expand_exits(world, ctx, spot_map, max_time, spot_heap);

    for warp in world.get_warps() {
        if !spot_map.contains_key(&warp.dest(ctx.get(), world)) && warp.can_access(ctx.get(), world)
        {
            let mut newctx = ctx.clone();
            newctx.warp(world, warp);
            let elapsed = newctx.elapsed();
            if elapsed <= max_time {
                spot_heap.push(Reverse(HeapElement {
                    score: elapsed,
                    el: newctx,
                }));
            }
        }
    }
}

fn expand_exits<W, T, E>(
    world: &W,
    ctx: &ContextWrapper<T>,
    spot_map: &HashMap<E::SpotId, ContextWrapper<T>, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<HeapElement<T>>>,
) where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
{
    for exit in world.get_spot_exits(ctx.get().position()) {
        if !spot_map.contains_key(&exit.dest()) && exit.can_access(ctx.get(), world) {
            let mut newctx = ctx.clone();
            newctx.exit(world, exit);
            let elapsed = newctx.elapsed();
            if elapsed <= max_time {
                spot_heap.push(Reverse(HeapElement {
                    score: elapsed,
                    el: newctx,
                }));
            }
        }
    }
}

// This is mainly for move_to which is used from tests.
fn expand_local<W, T, E, Wp>(
    world: &W,
    ctx: &ContextWrapper<T>,
    movement_state: T::MovementState,
    spot_map: &HashMap<E::SpotId, ContextWrapper<T>, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<HeapElement<T>>>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    for &dest in world.get_area_spots(ctx.get().position()) {
        let ltt = ctx.get().local_travel_time(movement_state, dest);
        if !spot_map.contains_key(&dest) && ltt < u32::MAX {
            let mut newctx = ctx.clone();
            newctx.move_local(world, dest, ltt);
            let elapsed = newctx.elapsed();
            if elapsed <= max_time {
                spot_heap.push(Reverse(HeapElement {
                    score: elapsed,
                    el: newctx,
                }));
            }
        }
    }
}

/// Explores outward from the current position.
pub fn accessible_spots<W, T, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_time: u32,
    allow_local: bool,
) -> HashMap<E::SpotId, ContextWrapper<T>, CommonHasher>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    W::Warp:
        Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    // return: spotid -> ctxwrapper
    let mut spot_enum_map = new_hashmap();
    let mut spot_heap = BinaryHeap::new();
    let pos = ctx.get().position();
    spot_enum_map.insert(pos, ctx);

    expand(
        world,
        &spot_enum_map[&pos],
        &spot_enum_map,
        max_time,
        &mut spot_heap,
        allow_local,
    );
    while let Some(Reverse(el)) = spot_heap.pop() {
        let spot_found = el.el;
        let pos = spot_found.get().position();
        if !spot_enum_map.contains_key(&pos) {
            spot_enum_map.insert(pos, spot_found);
            expand(
                world,
                &spot_enum_map[&pos],
                &spot_enum_map,
                max_time,
                &mut spot_heap,
                allow_local,
            );
        }
    }

    spot_enum_map
}

/// Finds the shortest route to the given spot, if any, and moves there.
pub fn move_to<W, T, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    spot: E::SpotId,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Result<ContextWrapper<T>, String>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    W::Warp:
        Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    if ctx.get().position() == spot {
        return Ok(ctx);
    }

    let goal = ExternalNodeId::Spot(spot);
    let score_func = |ctx: &ContextWrapper<T>| -> Option<u32> {
        let mut scores: Vec<Option<u32>> = vec![shortest_paths
            .min_distance(ExternalNodeId::Spot(ctx.get().position()), goal)
            .map(|u| u.try_into().unwrap())];
        // We need to take into account contextual warps which aren't otherwise part
        // of a normal shortest paths graph. We do that by measuring the shortest path
        // from their destination and adding in the warp time.
        // TODO: Only do this on contextual warps.
        for warp in world.get_warps() {
            scores.push(
                shortest_paths
                    .min_distance(ExternalNodeId::Spot(warp.dest(ctx.get(), world)), goal)
                    .map(|u| {
                        warp.time(ctx.get(), world) + <u64 as TryInto<u32>>::try_into(u).unwrap()
                    }),
            );
        }
        scores
            .into_iter()
            .filter_map(|u| u)
            .min()
            .map(|u| u + ctx.elapsed())
    };

    // Using A* and allowing backtracking
    let mut spot_heap =
        LimitedPriorityQueue::with_capacity_and_limit(INITIAL_CAPACITY, MAX_STATES_FOR_SPOTS);

    if let Some(score) = score_func(&ctx) {
        let unique_key = ctx.get().clone();
        spot_heap.push(ctx, unique_key, score);
    }

    while let Some((ctx, _)) = spot_heap.pop() {
        if ctx.get().position() == spot {
            return Ok(ctx);
        }
        expand_astar(
            world,
            &ctx,
            u32::MAX,
            &mut spot_heap,
            &score_func,
            W::same_area(ctx.get().position(), spot),
        );
    }

    Err(explain_unused_links(world, spot_heap.into_unique_key_map()))
}

pub fn nearest_location_by_heuristic<'w, W, T, L, E, I>(
    world: &W,
    ctx: &T,
    locs: I,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Option<&'w L>
where
    W: World<Exit = E, Location = L>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = L::Currency>,
    L: Location<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = L::Currency>,
    I: Iterator<Item = &'w L>,
{
    let mut origins = new_hashmap();
    origins.insert(ExternalNodeId::Spot(ctx.position()), 0);
    // We need to take into account contextual warps which aren't otherwise part
    // of a normal shortest paths graph. We do that by measuring the shortest path
    // from their destination and adding in the warp time.
    // TODO: Only do this on contextual warps.
    for warp in world.get_warps() {
        let dst = ExternalNodeId::Spot(warp.dest(ctx, world));
        let time = warp.time(ctx, world);
        if (!origins.contains_key(&dst) || time < origins[&dst]) && warp.can_access(ctx, world) {
            origins.insert(dst, time);
        }
    }

    locs.min_by_key(|loc| {
        let mut min = None;
        for (origin, warp_time) in &origins {
            let goal = ExternalNodeId::Canon(loc.canon_id());
            let time: Option<u32> = shortest_paths
                .min_distance(*origin, goal)
                .map(|u| u as u32 + loc.base_time() + warp_time);
            min = match (min, time) {
                (Some(m), Some(t)) => Some(std::cmp::min(m, t)),
                (None, Some(_)) => time,
                (_, None) => min,
            }
        }

        min.unwrap_or(u32::MAX)
    })
}

pub fn find_nearest_location_with_actions<W, T, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_time: u32,
    max_depth: usize,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Result<ContextWrapper<T>, String>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    W::Warp:
        Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    if world
        .get_spot_locations(ctx.get().position())
        .into_iter()
        .any(|loc| ctx.get().todo(loc) && loc.can_access(ctx.get(), world))
    {
        return Ok(ctx);
    }

    let mut todo_spots = new_hashmap();
    for loc in world.get_all_locations() {
        if ctx.get().todo(loc) {
            let spot_id = ExternalNodeId::Spot(world.get_location_spot(loc.id()));
            if let Some(spot_min) = todo_spots.get_mut(&spot_id) {
                *spot_min = std::cmp::min(*spot_min, loc.base_time());
            } else {
                todo_spots.insert(spot_id, loc.base_time());
            }
        }
    }
    if todo_spots.is_empty() {
        return Ok(ctx);
    }

    let score_func = |ctx: &ContextWrapper<T>| -> Option<u32> {
        let mut origins = new_hashmap();
        origins.insert(ExternalNodeId::Spot(ctx.get().position()), 0);
        // We need to take into account contextual warps which aren't otherwise part
        // of a normal shortest paths graph. We do that by measuring the shortest path
        // from their destination and adding in the warp time.
        // TODO: Only do this on contextual warps.
        for warp in world.get_warps() {
            let dst = ExternalNodeId::Spot(warp.dest(ctx.get(), world));
            let time = warp.time(ctx.get(), world);
            if (!origins.contains_key(&dst) || time < origins[&dst])
                && warp.can_access(ctx.get(), world)
            {
                origins.insert(dst, time);
            }
        }

        let mut min = None;
        for (origin, base_time) in origins {
            for (goal, min_time) in &todo_spots {
                let time: Option<u32> = shortest_paths
                    .min_distance(origin, *goal)
                    .map(|u| <u64 as TryInto<u32>>::try_into(u).unwrap() + *min_time + base_time);
                min = match (min, time) {
                    (Some(m), Some(t)) => Some(std::cmp::min(m, t)),
                    (None, Some(_)) => time,
                    (_, None) => min,
                }
            }
        }

        min
    };

    // Using A* and allowing backtracking
    let mut spot_heap =
        LimitedPriorityQueue::with_capacity_and_limit(INITIAL_CAPACITY, MAX_STATES_FOR_LOCS);

    if let Some(score) = score_func(&ctx) {
        let unique_key = ctx.get().clone();
        spot_heap.push(
            CtxWithActionCounter {
                el: ctx,
                counter: 0,
            },
            unique_key,
            score,
        );
    }

    while let Some((el, _)) = spot_heap.pop() {
        let ctx = &el.el;
        if world
            .get_spot_locations(ctx.get().position())
            .into_iter()
            .any(|loc| ctx.get().todo(loc) && loc.can_access(ctx.get(), world))
        {
            return Ok(el.el);
        }
        expand_astar(world, &el, max_time, &mut spot_heap, &score_func, false);

        if el.can_continue(max_depth) {
            expand_actions_astar(world, &el, max_time, &mut spot_heap, &score_func);
        }
    }

    Err(explain_unused_links(world, spot_heap.into_unique_key_map()))
}

fn access_check_after_actions<W, T, E, A, F, G>(
    world: &W,
    ctx: ContextWrapper<T>,
    spot: E::SpotId,
    check: &A,
    access: F,
    is_eligible: G,
    max_time: u32,
    max_depth: usize,
    max_states: usize,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Result<ContextWrapper<T>, String>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    W::Warp:
        Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
    A: Accessible<Context = T>,
    F: FnOnce(&mut ContextWrapper<T>, &W, &A),
    G: Fn(&T) -> bool,
{
    let goal = ExternalNodeId::Spot(spot);

    type DistanceScore = (u32, OrderedFloat<f32>);
    let score_func = |ctx: &ContextWrapper<T>| -> Option<DistanceScore> {
        if !shortest_paths
            .graph()
            .node_index_map
            .contains_key(&ExternalNodeId::Spot(ctx.get().position()))
        {
            panic!("SP Graph missing position: {}", ctx.get().position());
        }
        if !shortest_paths.graph().node_index_map.contains_key(&goal) {
            panic!("SP Graph missing goal: {}", spot);
        }
        let mut scores: Vec<Option<DistanceScore>> = vec![shortest_paths
            .min_distance(ExternalNodeId::Spot(ctx.get().position()), goal)
            .map(|u| {
                (
                    u.try_into().unwrap(),
                    OrderedFloat(W::spot_distance(ctx.get().position(), spot)),
                )
            })];
        // We need to take into account contextual warps which aren't otherwise part
        // of a normal shortest paths graph. We do that by measuring the shortest path
        // from their destination and adding in the warp time.
        // TODO: Only do this on contextual warps.
        for warp in world.get_warps() {
            scores.push(
                shortest_paths
                    .min_distance(ExternalNodeId::Spot(warp.dest(ctx.get(), world)), goal)
                    .map(|u| {
                        (
                            warp.time(ctx.get(), world)
                                + <u64 as TryInto<u32>>::try_into(u).unwrap(),
                            OrderedFloat(W::spot_distance(ctx.get().position(), spot)),
                        )
                    }),
            );
        }
        scores
            .into_iter()
            .filter_map(|u| u)
            .min()
            .map(|(u, f)| (u + ctx.elapsed(), f))
    };

    // Using A* and allowing backtracking
    let mut spot_heap = LimitedPriorityQueue::with_capacity_and_limit(
        std::cmp::min(INITIAL_CAPACITY, max_states),
        max_states,
    );

    if let Some(score) = score_func(&ctx) {
        let unique_key = ctx.get().clone();
        spot_heap.push(
            CtxWithActionCounter {
                el: ctx,
                counter: 0,
            },
            unique_key,
            score,
        );
    }

    while let Some((el, _)) = spot_heap.pop() {
        let ctx = &el.el;
        if is_eligible(ctx.get()) {
            let mut newctx = ctx.clone();
            access(&mut newctx, world, check);
            return Ok(newctx);
        }
        expand_astar(
            world,
            &el,
            u32::MAX,
            &mut spot_heap,
            &score_func,
            W::same_area(ctx.get().position(), spot),
        );

        if el.can_continue(max_depth) {
            expand_actions_astar(world, &el, max_time, &mut spot_heap, &score_func);
        }

        if spot_heap.is_expired() {
            return Err(format!(
                "Excessive A* search stopping at {} states explored",
                spot_heap.total_seen()
            ));
        }
    }

    Err(explain_unused_links(world, spot_heap.into_unique_key_map()))
}

pub fn access_location_after_actions<W, T, E, L>(
    world: &W,
    ctx: ContextWrapper<T>,
    loc_id: L,
    max_time: u32,
    max_depth: usize,
    max_states: usize,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Result<ContextWrapper<T>, String>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T, LocId = L>,
    W::Warp:
        Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
    L: Id,
{
    if ctx.get().visited(loc_id) {
        return Ok(ctx);
    }

    let spot = world.get_location_spot(loc_id);
    let loc = world.get_location(loc_id);

    access_check_after_actions(
        world,
        ctx,
        spot,
        loc,
        ContextWrapper::visit,
        |c| c.position() == spot && loc.can_access(c, world),
        max_time,
        max_depth,
        max_states,
        shortest_paths,
    )
}

pub fn access_action_after_actions<W, T, E, A>(
    world: &W,
    ctx: ContextWrapper<T>,
    act_id: A,
    max_time: u32,
    max_depth: usize,
    max_states: usize,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Result<ContextWrapper<T>, String>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    W::Action: Action<Context = T, ActionId = A, SpotId = E::SpotId>,
    W::Warp:
        Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
    A: Id,
{
    let spot = world.get_action_spot(act_id);
    assert!(
        spot != Default::default(),
        "access_after_actions not suitable for global actions"
    );
    let act = world.get_action(act_id);

    access_check_after_actions(
        world,
        ctx,
        spot,
        act,
        ContextWrapper::activate,
        |c| c.position() == spot && act.can_access(c, world),
        max_time,
        max_depth,
        max_states,
        shortest_paths,
    )
}

pub fn all_visitable_locations<W, T, L, E>(world: &W, ctx: &T) -> Vec<L::LocId>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    E: Exit<Context = T>,
{
    world
        .get_spot_locations(ctx.position())
        .iter()
        .filter_map(|loc| {
            if ctx.todo(loc) && loc.can_access(ctx, world) {
                Some(loc.id())
            } else {
                None
            }
        })
        .collect()
}

pub fn can_win<W, T, L, E>(world: &W, ctx: &T, max_time: u32) -> Result<(), ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let res = greedy_search_from(world, ctx, max_time);
    match res {
        Ok(_) => Ok(()),
        Err(c) => Err(c),
    }
}

pub fn can_win_just_items<W, T, L, E>(world: &W, ctx: &T) -> Result<(), Vec<(T::ItemId, i16)>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let mut ctx = ctx.clone();
    for loc in world.get_all_locations() {
        if ctx.todo(loc) {
            ctx.visit(loc.id());
            ctx.collect(loc.item(), world);
        }
    }
    if world.won(&ctx) {
        Ok(())
    } else {
        Err(world.items_needed(&ctx))
    }
}

pub fn can_win_just_locations<W, T, L, E>(world: &W, ctx: &T) -> Result<(), Vec<(T::ItemId, i16)>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let mut ctx = ctx.clone();
    let mut found = true;
    while found {
        found = false;
        for loc in world.get_all_locations() {
            if ctx.todo(loc) && loc.can_access(&ctx, world) {
                ctx.visit(loc.id());
                ctx.collect(loc.item(), world);
                found = true;
            }
        }
        if world.won(&ctx) {
            return Ok(());
        }
    }
    Err(world.items_needed(&ctx))
}

pub fn find_unused_links<W, T, E, Wp>(
    world: &W,
    spot_map: &HashMap<E::SpotId, ContextWrapper<T>, CommonHasher>,
) -> String
where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    let mut accessible: Vec<_> = spot_map.values().collect();
    accessible.sort_unstable_by_key(|el| el.elapsed());
    let mut vec = Vec::new();
    for ctx in accessible {
        for spot in world.get_area_spots(ctx.get().position()) {
            if !spot_map.contains_key(spot)
                && world
                    .get_condensed_edges_from(ctx.get().position())
                    .into_iter()
                    .any(|ce| ce.dst == *spot)
            {
                vec.push(format!(
                    "{} -> {}: movement not available",
                    ctx.get().position(),
                    spot
                ));
            }
        }

        for exit in world.get_spot_exits(ctx.get().position()) {
            if !spot_map.contains_key(&exit.dest()) {
                vec.push(format!(
                    "{}: exit not usable:\n{}",
                    exit.id(),
                    exit.explain(ctx.get(), world)
                ));
            }
        }
    }
    vec.join("\n")
}

fn explain_unused_links<W, T, E, Wp, P>(
    world: &W,
    states_seen: HashMap<T, P, CommonHasher>,
) -> String
where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
{
    let known_spots: HashSet<E::SpotId, CommonHasher> =
        states_seen.iter().map(|(c, _)| c.position()).collect();
    let mut vec = Vec::new();
    for (ctx, _) in states_seen {
        let movements = ctx.get_movement_state(world);
        for spot in world.get_area_spots(ctx.position()) {
            if !known_spots.contains(spot) {
                for ce in world.get_condensed_edges_from(ctx.position()) {
                    if ce.dst == *spot {
                        vec.push(format!(
                            "CE not available from {}: {:?}\n{}",
                            ctx.position(),
                            ce,
                            ce.explain(world, &ctx, movements)
                        ));
                    }
                }
            }
        }

        for exit in world.get_spot_exits(ctx.position()) {
            if !known_spots.contains(&exit.dest()) {
                vec.push(format!(
                    "{}: exit not usable:\n{}",
                    exit.id(),
                    exit.explain(&ctx, world)
                ));
            }
        }
    }
    if vec.len() > 20 {
        let excess = vec.len() - 20;
        vec.truncate(20);
        vec.push(format!("...and {} more", excess));
    }
    vec.join("\n")
}
