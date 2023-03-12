//! Functions related to access graphs, and accessing locations.

use crate::context::*;
use crate::world::*;
use std::cmp::Reverse;
use std::collections::VecDeque;
use std::collections::{BinaryHeap, HashMap};

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
        .any(|act| act.can_access(ctx.get()) && ctx.is_useful(act))
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
    spot_map: &HashMap<E::SpotId, ContextWrapper<T>>,
    spot_heap: &mut BinaryHeap<Reverse<ContextWrapper<T>>>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    for spot in world.get_area_spots(ctx.get().position()) {
        if !spot_map.contains_key(spot) {
            let local = ctx.get().local_travel_time(*spot);
            if local < 0 {
                // Can't move this way
                continue;
            }
            let mut newctx = ctx.clone();
            newctx.move_local(*spot, local);
            spot_heap.push(Reverse(newctx));
        }
    }

    for exit in world.get_spot_exits(ctx.get().position()) {
        if !spot_map.contains_key(&exit.dest()) && exit.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.exit(exit);
            spot_heap.push(Reverse(newctx));
        }
    }

    for warp in world.get_warps() {
        if !spot_map.contains_key(&warp.dest(ctx.get())) && warp.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.warp(warp);
            spot_heap.push(Reverse(newctx));
        }
    }
}

/// Variant for expand which doesn't track history or time, ideal for beatability checks
// (Is it possible to combine with the other without making it slower?)
pub fn expand_simple<W, T, E, Wp>(
    world: &W,
    ctx: &T,
    spot_map: &HashMap<E::SpotId, T>,
    ctx_queue: &mut VecDeque<T>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId>,
{
    let mut append = |spot| {
        // We're copying the whole context on every step, which is probably
        // super inefficient; we only really copy because position may be relevant
        // for connection checks. If we tracked the "context" state separately
        // from the item state, it might be less copying.
        let mut newctx = ctx.clone();
        newctx.set_position(spot);
        ctx_queue.push_back(newctx);
    };
    for spot in world.get_area_spots(ctx.position()) {
        if !spot_map.contains_key(spot) {
            let local = ctx.local_travel_time(*spot);
            if local < 0 {
                // Can't move this way
                continue;
            }
            append(*spot);
        }
    }

    for exit in world.get_spot_exits(ctx.position()) {
        if !spot_map.contains_key(&exit.dest()) && exit.can_access(ctx) {
            append(exit.dest());
        }
    }

    for warp in world.get_warps() {
        if !spot_map.contains_key(&warp.dest(ctx)) && warp.can_access(ctx) {
            let mut newctx = ctx.clone();
            warp.prewarp(&mut newctx);
            newctx.set_position(warp.dest(&newctx));
            ctx_queue.push_back(newctx);
        }
    }
}

/// Explores outward from the current position.
pub fn accessible_spots<W, T, E>(
    world: &W,
    ctx: ContextWrapper<T>,
) -> HashMap<E::SpotId, ContextWrapper<T>>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Warp:
        Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    // return: spotid -> ctxwrapper
    let mut spot_map = HashMap::new();
    let mut spot_heap = BinaryHeap::new();
    let pos = ctx.get().position();
    spot_map.insert(pos, ctx);

    expand(world, &spot_map[&pos], &spot_map, &mut spot_heap);
    while let Some(Reverse(spot_found)) = spot_heap.pop() {
        let pos = spot_found.get().position();
        if !spot_map.contains_key(&pos) {
            spot_map.insert(pos, spot_found);
            expand(world, &spot_map[&pos], &spot_map, &mut spot_heap);
        }
    }

    // TODO: sort by distance
    spot_map
}

/// Variant of `access` that does not write hist or time, ideal for beatability checks
pub fn access_simple<W, T, E>(world: &W, ctx: &T) -> HashMap<<E as Exit>::SpotId, T>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId>,
{
    let pos = ctx.position();
    let mut spot_map = HashMap::new();
    let mut ctx_queue = VecDeque::new();
    spot_map.insert(pos, ctx.clone());

    expand_simple(world, &spot_map[&pos], &spot_map, &mut ctx_queue);
    while !ctx_queue.is_empty() {
        let spot_found = ctx_queue.pop_front().unwrap();
        let pos = spot_found.position();
        if !spot_map.contains_key(&pos) {
            spot_map.insert(pos, spot_found);
            expand_simple(world, &spot_map[&pos], &spot_map, &mut ctx_queue);
        }
    }

    spot_map
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

pub fn visit_simple<W, T, L, E>(world: &W, ctx: &mut T) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
{
    let mut ret = false;
    for (spot_id, spot_ctx) in access_simple(world, &ctx) {
        for act in world.get_spot_actions(spot_id) {
            if act.can_access(&spot_ctx) && act.has_effect(&spot_ctx) {
                act.perform(ctx);
            }
        }
        for loc in world.get_spot_locations(spot_id) {
            // Check can_access at the local spot_ctx, but pick up items
            // and perform other checks with the omnipresent context.
            if ctx.todo(loc.id()) && loc.can_access(&spot_ctx) && ctx.can_afford(loc.price()) {
                ctx.collect(loc.item());
                ctx.spend(loc.price());
                for canon_loc_id in world.get_canon_locations(loc.id()) {
                    if ctx.todo(canon_loc_id) {
                        ctx.skip(canon_loc_id);
                    }
                }
                ctx.visit(loc.id());
                ret = true;
            }
        }
    }
    ret
}

pub fn can_win<W, T, L, E>(world: &W, ctx: &T) -> Result<(), T>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
{
    let mut ctx = ctx.clone();
    let mut acts_only = 0;
    while !world.won(&ctx) {
        if !visit_simple(world, &mut ctx) {
            acts_only += 1;
        } else {
            acts_only = 0;
        }
        if acts_only > 1 {
            return Err(ctx);
        }
    }
    Ok(())
}

pub fn cant_win<W, T, L, E>(world: &W, ctx: &T) -> Result<(), T>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
{
    let mut ctx = ctx.clone();
    let mut acts_only = 0;
    while !world.won(&ctx) {
        if !visit_simple(world, &mut ctx) {
            acts_only += 1;
        } else {
            acts_only = 0;
        }
        if acts_only > 1 {
            return Ok(());
        }
    }
    Err(ctx)
}

pub fn find_unused_links<W, T, E>(
    world: &W,
    spot_map: &HashMap<E::SpotId, ContextWrapper<T>>,
) -> String
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    let mut accessible: Vec<ContextWrapper<T>> = spot_map.clone().into_values().collect();
    accessible.sort_unstable_by_key(|el| el.elapsed());
    let mut vec = Vec::new();
    for ctx in accessible {
        for spot in world.get_area_spots(ctx.get().position()) {
            if !spot_map.contains_key(spot) && world.are_spots_connected(ctx.get().position(), *spot) {
                vec.push(format!("{} -> {}: movement not available", ctx.get().position(), spot));
            }
        }

        for exit in world.get_spot_exits(ctx.get().position()) {
            if !spot_map.contains_key(&exit.dest()) {
                vec.push(format!("{}: exit not usable", exit.id()));
            }
        }

        for warp in world.get_warps() {
            if !spot_map.contains_key(&warp.dest(ctx.get())) {
                vec.push(format!("{}: warp {} not usable", ctx.get().position(), warp.id()));
            }
        }
    }
    vec.join("\n")
}
