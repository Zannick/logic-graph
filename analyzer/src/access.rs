//! Functions related to access graphs, and accessing locations.

use enum_map::EnumMap;

use crate::context::*;
use crate::greedy::greedy_search_from;
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
pub fn spot_has_actions<W, T, L, E>(world: &W, ctx: &ContextWrapper<T>) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
{
    world
        .get_global_actions()
        .iter()
        .chain(world.get_spot_actions(ctx.get().position()))
        .any(|act| act.can_access(ctx.get()))
}

pub fn spot_has_locations_or_actions<W, T, L, E>(world: &W, ctx: &ContextWrapper<T>) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
{
    spot_has_locations(world, ctx.get()) || spot_has_actions(world, ctx)
}

pub fn expand<W, T, E, Wp>(
    world: &W,
    ctx: &ContextWrapper<T>,
    spot_map: &EnumMap<E::SpotId, Option<ContextWrapper<T>>>,
    max_time: i32,
    spot_heap: &mut BinaryHeap<Reverse<ContextWrapper<T>>>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    let movement_state = ctx.get().get_movement_state();
    for spot in world.get_area_spots(ctx.get().position()) {
        if spot_map[*spot] == None {
            let local = ctx.get().local_travel_time(movement_state, *spot);
            if local < 0 {
                // Can't move this way
                continue;
            }
            let mut newctx = ctx.clone();
            newctx.move_local(*spot, local);
            if newctx.elapsed() <= max_time {
                spot_heap.push(Reverse(newctx));
            }
        }
    }

    for exit in world.get_spot_exits(ctx.get().position()) {
        if spot_map[exit.dest()] == None && exit.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.exit(exit);
            if newctx.elapsed() <= max_time {
                spot_heap.push(Reverse(newctx));
            }
        }
    }

    for warp in world.get_warps() {
        if spot_map[warp.dest(ctx.get())] == None && warp.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.warp(warp);
            if newctx.elapsed() <= max_time {
                spot_heap.push(Reverse(newctx));
            }
        }
    }
}

/// Explores outward from the current position.
pub fn accessible_spots<W, T, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_time: i32,
) -> EnumMap<E::SpotId, Option<ContextWrapper<T>>>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Warp:
        Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    // return: spotid -> ctxwrapper
    let mut spot_enum_map: EnumMap<E::SpotId, Option<ContextWrapper<T>>> = EnumMap::default();
    let mut spot_heap = BinaryHeap::new();
    let pos = ctx.get().position();
    spot_enum_map[pos] = Some(ctx);

    expand(
        world,
        spot_enum_map[pos].as_ref().unwrap(),
        &spot_enum_map,
        max_time,
        &mut spot_heap,
    );
    while let Some(Reverse(spot_found)) = spot_heap.pop() {
        let pos = spot_found.get().position();
        if spot_enum_map[pos] == None {
            spot_enum_map[pos] = Some(spot_found);
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

pub fn all_visitable_locations<'a, W, T, L, E>(world: &'a W, ctx: &T) -> Vec<L::LocId>
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

pub fn visitable_locations<'a, W, T, L, E>(
    world: &'a W,
    ctx: &T,
) -> (Vec<&'a L>, Option<(<L as Location>::LocId, E::ExitId)>)
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
                if exit == None {
                    exit = Some((loc.id(), *e));
                }
                return false;
            }
            true
        })
        .collect();
    (locs, exit)
}

pub fn can_win<W, T, L, E>(world: &W, ctx: &T, max_time: i32) -> Result<(), ContextWrapper<T>>
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

pub fn can_win_just_items<W, T, L, E>(world: &W, ctx: &T) -> bool
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
    world.won(&ctx)
}

pub fn can_win_just_locations<W, T, L, E>(world: &W, ctx: &T) -> bool
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
            return true;
        }
    }
    false
}

pub fn find_unused_links<W, T, E, Wp>(
    world: &W,
    spot_map: &EnumMap<E::SpotId, Option<ContextWrapper<T>>>,
) -> String
where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    let mut accessible: Vec<ContextWrapper<T>> =
        spot_map.values().filter_map(Clone::clone).collect();
    accessible.sort_unstable_by_key(|el| el.elapsed());
    let mut vec = Vec::new();
    for ctx in accessible {
        for spot in world.get_area_spots(ctx.get().position()) {
            if spot_map[*spot] == None && world.are_spots_connected(ctx.get().position(), *spot) {
                vec.push(format!(
                    "{} -> {}: movement not available",
                    ctx.get().position(),
                    spot
                ));
            }
        }

        for exit in world.get_spot_exits(ctx.get().position()) {
            if spot_map[exit.dest()] == None {
                vec.push(format!("{}: exit not usable", exit.id()));
            }
        }

        for warp in world.get_warps() {
            if spot_map[warp.dest(ctx.get())] == None {
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
