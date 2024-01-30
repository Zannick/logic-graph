//! Functions related to access graphs, and accessing locations.

use crate::a_star::*;
use crate::context::*;
use crate::greedy::greedy_search_from;
use crate::heap::HeapElement;
use crate::steiner::graph::ExternalNodeId;
use crate::steiner::SteinerAlgo;
use crate::steiner::{loc_to_graph_node, EdgeId, NodeId, ShortestPaths};
use crate::world::*;
use crate::{new_hashmap, new_hashset, CommonHasher};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Check whether there are available locations at this position.
pub fn spot_has_locations<W, T, L, E>(world: &W, ctx: &T) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
{
    world
        .get_spot_locations(ctx.position())
        .iter()
        .any(|loc| ctx.todo(loc.id()) && loc.can_access(ctx, world))
}

/// Check whether there are available actions at this position, including global actions.
pub fn spot_has_actions<W, T, L, E>(world: &W, ctx: &T) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
{
    world
        .get_global_actions()
        .iter()
        .chain(world.get_spot_actions(ctx.position()))
        .any(|act| act.can_access(ctx, world))
}

pub fn spot_has_locations_or_actions<W, T, L, E>(world: &W, ctx: &ContextWrapper<T>) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
{
    spot_has_locations(world, ctx.get()) || spot_has_actions(world, ctx.get())
}

fn expand<W, T, E, Wp>(
    world: &W,
    ctx: &ContextWrapper<T>,
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
    let mut states_seen = new_hashset();
    let mut spot_heap = BinaryHeap::new();

    if let Some(score) = score_func(&ctx) {
        spot_heap.push(Reverse(HeapElement { score, el: ctx }));
    }

    while let Some(Reverse(el)) = spot_heap.pop() {
        let ctx = &el.el;
        if ctx.get().position() == spot {
            return Ok(el.el);
        }
        if !states_seen.insert(ctx.get().clone()) {
            continue;
        }
        expand_astar(
            world,
            &el,
            &mut states_seen,
            u32::MAX,
            &mut spot_heap,
            &score_func,
            W::same_area(ctx.get().position(), spot),
        );
    }

    Err(explain_unused_links(world, &states_seen))
}

pub fn find_nearest_location_with_actions<W, T, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_time: u32,
    max_depth: i8,
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
        .any(|loc| ctx.get().todo(loc.id()) && loc.can_access(ctx.get(), world))
    {
        return Ok(ctx);
    }

    let mut todo_spots = new_hashmap();
    for loc in world.get_all_locations() {
        if ctx.get().todo(loc.id()) {
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
    let mut states_seen = new_hashset();
    let mut spot_heap = BinaryHeap::new();

    if let Some(score) = score_func(&ctx) {
        spot_heap.push(Reverse(ScoredCtxWithActionCounter {
            score,
            el: ctx,
            counter: 0,
        }));
    }

    while let Some(Reverse(el)) = spot_heap.pop() {
        let ctx = &el.el;
        if world
            .get_spot_locations(ctx.get().position())
            .into_iter()
            .any(|loc| ctx.get().todo(loc.id()) && loc.can_access(ctx.get(), world))
        {
            return Ok(el.el);
        }
        if !states_seen.insert(ctx.get().clone()) {
            continue;
        }
        expand_astar(
            world,
            &el,
            &mut states_seen,
            max_time,
            &mut spot_heap,
            &score_func,
            false,
        );

        if el.can_continue(max_depth) {
            expand_actions_astar(
                world,
                &el,
                &mut states_seen,
                max_time,
                &mut spot_heap,
                &score_func,
            );
        }
    }

    Err(explain_unused_links(world, &states_seen))
}

pub fn access_location_after_actions<W, T, E, L>(
    world: &W,
    ctx: ContextWrapper<T>,
    loc_id: L,
    max_time: u32,
    max_depth: i8,
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
    } else if ctx.get().skipped(loc_id) {
        return Err(format!("Location already skipped: {}", loc_id));
    }

    let goal = loc_to_graph_node(world, loc_id);
    let score_func = |ctx: &ContextWrapper<T>| -> Option<u32> {
        if !shortest_paths.graph().node_index_map.contains_key(&ExternalNodeId::Spot(ctx.get().position())) {
            panic!("SP Graph missing position: {}", ctx.get().position());
        }
        if !shortest_paths.graph().node_index_map.contains_key(&goal) {
            panic!("SP Graph missing goal: {}", loc_id);
        }
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
    let mut states_seen = new_hashset();
    let mut spot_heap = BinaryHeap::new();

    if let Some(score) = score_func(&ctx) {
        spot_heap.push(Reverse(ScoredCtxWithActionCounter {
            score,
            el: ctx,
            counter: 0,
        }));
    }

    let spot = world.get_location_spot(loc_id);
    let loc = world.get_location(loc_id);
    while let Some(Reverse(el)) = spot_heap.pop() {
        let ctx = &el.el;
        if ctx.get().position() == spot && loc.can_access(ctx.get(), world) {
            let mut newctx = ctx.clone();
            newctx.visit(world, loc);
            return Ok(newctx);
        }
        if !states_seen.insert(ctx.get().clone()) {
            continue;
        }
        expand_astar(
            world,
            &el,
            &mut states_seen,
            u32::MAX,
            &mut spot_heap,
            &score_func,
            W::same_area(ctx.get().position(), spot),
        );

        if el.can_continue(max_depth) {
            expand_actions_astar(
                world,
                &el,
                &mut states_seen,
                max_time,
                &mut spot_heap,
                &score_func,
            );
        }

        if states_seen.len() > world.get_all_spots().len() * 3 {
            return Err(format!(
                "Excessive A* search stopping at {} states explored, {} left in queue",
                states_seen.len(),
                spot_heap.len()
            ));
        }
    }

    Err(explain_unused_links(world, &states_seen))
}

pub fn all_visitable_locations<W, T, L, E>(world: &W, ctx: &T) -> Vec<L::LocId>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
{
    world
        .get_spot_locations(ctx.position())
        .iter()
        .filter_map(|loc| {
            if ctx.todo(loc.id()) && loc.can_access(ctx, world) {
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
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
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
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let mut ctx = ctx.clone();
    world.skip_unused_items(&mut ctx);
    for loc in world.get_all_locations() {
        if ctx.todo(loc.id()) {
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
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let mut ctx = ctx.clone();
    world.skip_unused_items(&mut ctx);
    let mut found = true;
    while found {
        found = false;
        for loc in world.get_all_locations() {
            if ctx.todo(loc.id()) && loc.can_access(&ctx, world) {
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
            if !spot_map.contains_key(&exit.dest())
                && (!W::same_area(ctx.get().position(), exit.dest())
                    || world
                        .get_condensed_edges_from(ctx.get().position())
                        .into_iter()
                        .any(|ce| ce.dst == exit.dest()))
            {
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

pub fn explain_unused_links<W, T, E, Wp>(
    world: &W,
    states_seen: &HashSet<T, CommonHasher>,
) -> String
where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
{
    let known_spots: HashSet<E::SpotId, CommonHasher> =
        states_seen.iter().map(|c| c.position()).collect();
    let mut vec = Vec::new();
    for ctx in states_seen {
        let movements = ctx.get_movement_state(world);
        for spot in world.get_area_spots(ctx.position()) {
            if !known_spots.contains(spot) {
                for ce in world.get_condensed_edges_from(ctx.position()) {
                    if ce.dst == *spot {
                        vec.push(format!(
                            "CE not available from {}: {:?}\n{}",
                            ctx.position(),
                            ce,
                            ce.explain(world, ctx, movements)
                        ));
                    }
                }
            }
        }

        for exit in world.get_spot_exits(ctx.position()) {
            if !known_spots.contains(&exit.dest())
                && (!W::same_area(ctx.position(), exit.dest())
                    || world
                        .get_condensed_edges_from(ctx.position())
                        .into_iter()
                        .any(|ce| ce.dst == exit.dest()))
            {
                vec.push(format!(
                    "{}: exit not usable:\n{}",
                    exit.id(),
                    exit.explain(&ctx, world)
                ));
            }
        }
    }
    vec.join("\n")
}
