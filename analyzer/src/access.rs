//! Functions related to access graphs, and accessing locations.

use enum_map::EnumMap;

use crate::context::*;
use crate::greedy::greedy_search_from;
use crate::heap::HeapElement;
use crate::world::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

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
        .any(|loc| ctx.todo(loc.id()) && loc.can_access(ctx))
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
        .any(|act| act.can_access(ctx))
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
    spot_map: &EnumMap<E::SpotId, Option<Box<ContextWrapper<T>>>>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<HeapElement<T>>>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    let movement_state = ctx.get().get_movement_state();
    for ce in world.get_condensed_edges_from(ctx.get().position()) {
        if spot_map[ce.dst].is_none() && ce.can_access(world, ctx.get(), movement_state) {
            let mut newctx = ctx.clone();
            newctx.move_condensed_edge(ce);
            let elapsed = newctx.elapsed();
            if elapsed <= max_time {
                spot_heap.push(Reverse(HeapElement {
                    score: elapsed,
                    el: newctx,
                }));
            }
        }
    }

    for exit in world.get_spot_exits(ctx.get().position()) {
        if spot_map[exit.dest()].is_none() && exit.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.exit(exit);
            let elapsed = newctx.elapsed();
            if elapsed <= max_time {
                spot_heap.push(Reverse(HeapElement {
                    score: elapsed,
                    el: newctx,
                }));
            }
        }
    }

    for warp in world.get_warps() {
        if spot_map[warp.dest(ctx.get())].is_none() && warp.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.warp(warp);
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
fn expand_with_local<W, T, E, Wp>(
    world: &W,
    ctx: &ContextWrapper<T>,
    spot_map: &EnumMap<E::SpotId, Option<Box<ContextWrapper<T>>>>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<HeapElement<T>>>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    expand(world, ctx, spot_map, max_time, spot_heap);
    let movement_state = ctx.get().get_movement_state();
    for &dest in world.get_area_spots(ctx.get().position()) {
        let ltt = ctx.get().local_travel_time(movement_state, dest);
        if spot_map[dest].is_none() && ltt < u32::MAX {
            let mut newctx = ctx.clone();
            newctx.move_local(dest, ltt);
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
) -> EnumMap<E::SpotId, Option<Box<ContextWrapper<T>>>>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    W::Warp:
        Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    // return: spotid -> ctxwrapper
    let mut spot_enum_map: EnumMap<E::SpotId, Option<Box<ContextWrapper<T>>>> = EnumMap::default();
    let mut spot_heap = BinaryHeap::new();
    let pos = ctx.get().position();
    spot_enum_map[pos] = Some(Box::new(ctx));

    expand(
        world,
        spot_enum_map[pos].as_ref().unwrap(),
        &spot_enum_map,
        max_time,
        &mut spot_heap,
    );
    while let Some(Reverse(el)) = spot_heap.pop() {
        let spot_found = el.el;
        let pos = spot_found.get().position();
        if spot_enum_map[pos].is_none() {
            spot_enum_map[pos] = Some(Box::new(spot_found));
            expand(
                world,
                spot_enum_map[pos].as_ref().unwrap(),
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
) -> Option<ContextWrapper<T>>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    W::Warp:
        Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    if ctx.get().position() == spot {
        return Some(ctx);
    }
    let mut spot_enum_map: EnumMap<E::SpotId, Option<Box<ContextWrapper<T>>>> = EnumMap::default();
    let mut spot_heap = BinaryHeap::new();
    let pos = ctx.get().position();
    spot_enum_map[pos] = Some(Box::new(ctx));

    expand_with_local(
        world,
        spot_enum_map[pos].as_ref().unwrap(),
        &spot_enum_map,
        u32::MAX,
        &mut spot_heap,
    );

    while let Some(Reverse(el)) = spot_heap.pop() {
        let spot_found = el.el;
        let pos = spot_found.get().position();
        if pos == spot {
            return Some(spot_found);
        }
        if spot_enum_map[pos].is_none() {
            spot_enum_map[pos] = Some(Box::new(spot_found));
            expand_with_local(
                world,
                spot_enum_map[pos].as_ref().unwrap(),
                &spot_enum_map,
                u32::MAX,
                &mut spot_heap,
            );
        }
    }
    None
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
            if ctx.todo(loc.id()) && loc.can_access(ctx) {
                Some(loc.id())
            } else {
                None
            }
        })
        .collect()
}

pub struct ExitWithLoc<L: Location, E: Exit>(pub <L as Location>::LocId, pub <E as Exit>::ExitId);
pub fn visitable_locations<'a, W, T, L, E>(
    world: &'a W,
    ctx: &T,
) -> (Vec<&'a L>, Option<ExitWithLoc<L, E>>)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
{
    let mut exit = None;
    let locs: Vec<&L> = world
        .get_spot_locations(ctx.position())
        .iter()
        .filter(|loc| {
            if !ctx.todo(loc.id()) || !loc.can_access(ctx) {
                return false;
            } else if let Some(e) = loc.exit_id() {
                if exit.is_none() {
                    exit = Some(ExitWithLoc(loc.id(), *e));
                }
                return false;
            }
            true
        })
        .collect();
    (locs, exit)
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
            ctx.collect(loc.item());
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
            if ctx.todo(loc.id()) && loc.can_access(&ctx) {
                ctx.visit(loc.id());
                ctx.collect(loc.item());
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
    spot_map: &EnumMap<E::SpotId, Option<Box<ContextWrapper<T>>>>,
) -> String
where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    let mut accessible: Vec<Box<ContextWrapper<T>>> =
        spot_map.values().filter_map(Clone::clone).collect();
    accessible.sort_unstable_by_key(|el| el.elapsed());
    let mut vec = Vec::new();
    for ctx in accessible {
        for spot in world.get_area_spots(ctx.get().position()) {
            if spot_map[*spot].is_none() && world.get_condensed_edges_from(ctx.get().position()).into_iter().any(|ce| ce.dst == *spot) {
                vec.push(format!(
                    "{} -> {}: movement not available",
                    ctx.get().position(),
                    spot
                ));
            }
        }

        for exit in world.get_spot_exits(ctx.get().position()) {
            if spot_map[exit.dest()].is_none() && (!W::same_area(ctx.get().position(), exit.dest()) || world.get_condensed_edges_from(ctx.get().position()).into_iter().any(|ce| ce.dst == exit.dest())) {
                vec.push(format!("{}: exit not usable", exit.id()));
            }
        }

        for warp in world.get_warps() {
            if spot_map[warp.dest(ctx.get())].is_none() {
                vec.push(format!(
                    "{}: warp {} not usable",
                    ctx.get().position(),
                    warp.id()
                ));
            }
        }
    }
    vec.join("\n")
}
