use crate::context::*;
use crate::world::*;
use std::cmp::Reverse;
use std::collections::VecDeque;
use std::collections::{BinaryHeap, HashMap};

pub fn spot_has_locations<'a, W, T, L, E>(world: &'a W, ctx: &T, spot: E::SpotId) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context<'a> = T>,
    E: Exit<Context<'a> = T>,
{
    world
        .get_spot_locations(spot)
        .iter()
        .any(|loc| ctx.todo(loc.id()) && loc.can_access(ctx))
}

pub fn spot_has_actions<'a, W, T, L, E>(world: &'a W, ctx: &T, spot: E::SpotId) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context<'a> = T>,
    E: Exit<Context<'a> = T>,
{
    world
        .get_spot_actions(spot)
        .iter()
        .any(|act| act.can_access(ctx))
}

pub fn spot_has_locations_or_actions<'a, W, T, L, E>(world: &'a W, ctx: &T, spot: E::SpotId) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context<'a> = T>,
    E: Exit + Accessible<Context<'a> = T>,
{
    spot_has_locations(world, ctx, spot) || spot_has_actions(world, ctx, spot)
}

pub fn expand<'a, W, T, E, Wp>(
    world: &'a W,
    ctx: &ContextWrapper<T>,
    dist_map: &HashMap<E::SpotId, ContextWrapper<T>>,
    spot_heap: &mut BinaryHeap<Reverse<ContextWrapper<T>>>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context<'a> = T>,
    Wp: Warp<Context<'a> = T, SpotId = E::SpotId>,
{
    let mut insert = |spot, time, hist| {
        // We're copying the whole context on every step, which is probably
        // super inefficient; we only really copy because position may be relevant
        // for connection checks. If we tracked the "context" state separately
        // from the item state, it might be less copying.
        let mut newctx = ctx.clone();
        newctx.get_mut().set_position(spot);
        newctx.history.push(hist);
        newctx.elapse(time);
        spot_heap.push(Reverse(newctx));
    };

    for spot in world.get_area_spots(ctx.get().position()) {
        if !dist_map.contains_key(spot) {
            let local = ctx.get().local_travel_time(*spot);
            if local < 0 {
                panic!(
                    "Could not travel within area: start={:?} dest={:?}",
                    ctx.get().position(),
                    spot
                );
            }
            insert(*spot, local, History::MoveLocal(*spot));
        }
    }

    for exit in world.get_spot_exits(ctx.get().position()) {
        if !dist_map.contains_key(&exit.dest()) && exit.can_access(ctx.get()) {
            insert(exit.dest(), exit.time(), History::Move(exit.id()));
        }
    }

    for warp in world.get_warps() {
        if !dist_map.contains_key(&warp.dest(ctx.get())) && warp.can_access(ctx.get()) {
            insert(
                warp.dest(ctx.get()),
                warp.time(),
                History::Warp(warp.id(), warp.dest(ctx.get())),
            );
        }
    }
}

/// Variant for expand which doesn't track history or time, ideal for beatability checks
// (Is it possible to combine with the other without making it slower?)
pub fn expand_simple<'a, W, T, E, Wp>(
    world: &W,
    ctx: &T,
    spot_map: &HashMap<E::SpotId, T>,
    ctx_queue: &mut VecDeque<T>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context<'a> = T>,
    Wp: Warp<Context<'a> = T, SpotId = E::SpotId>,
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
            append(warp.dest(ctx));
        }
    }
}

// At some point I should add counting of attempts
pub fn access<'a, W, T, E>(world: &'a W, ctx: ContextWrapper<T>) -> HashMap<E::SpotId, ContextWrapper<T>>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context<'a> = T>,
    W::Warp: Warp<Context<'a> = T, SpotId = E::SpotId>,
{
    // return: spotid -> ctxwrapper
    let mut dist_map = HashMap::new();
    let mut spot_heap = BinaryHeap::new();
    let pos = ctx.get().position();
    dist_map.insert(pos, ctx);

    expand(world, &dist_map[&pos], &dist_map, &mut spot_heap);
    while !spot_heap.is_empty() {
        let spot_found = spot_heap.pop().unwrap().0;
        let pos = spot_found.get().position();
        if !dist_map.contains_key(&pos) {
            dist_map.insert(pos, spot_found);
            expand(world, &dist_map[&pos], &dist_map, &mut spot_heap);
        }
    }

    // TODO: sort by distance
    dist_map
}

/// Variant of `access` that does not write hist or time, ideal for beatability checks
pub fn access_simple<'a, W, T, E>(world: &W, ctx: &T) -> HashMap<<E as Exit>::SpotId, T>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context<'a> = T>,
    W::Warp: Warp<Context<'a> = T, SpotId = E::SpotId>,
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

pub fn visitable_locations<'a, W, T, L, E>(
    world: &'a W,
    ctx: &T,
) -> (Vec<&'a L>, Option<(<L as Location>::LocId, E::ExitId)>)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context<'a> = T>,
    E: Exit + Accessible<Context<'a> = T>,
{
    let mut exit = None;
    let locs: Vec<&L> = world
        .get_spot_locations(ctx.position())
        .iter()
        .filter(|loc| {
            if !ctx.todo(loc.id()) || !loc.can_access(ctx) {
                return false;
            } else if exit == None {
                if let Some(e) = loc.exit_id() {
                    exit = Some((loc.id(), *e));
                    return false;
                }
            }
            true
        })
        .collect();
    (locs, exit)
}

pub fn visit_fanout<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    allow_skips: bool,
) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + for<'a> Accessible<Context<'a> = T>,
    E: Exit + for<'a> Accessible<Context<'a> = T>,
{
    let mut ctx_list = vec![ctx];
    for act in world.get_spot_actions(ctx_list[0].get().position()) {
        if act.can_access(ctx_list[0].get()) {
            ctx_list[0].activate(act);
        }
    }
    let (locs, exit) = visitable_locations(world, ctx_list[0].get());

    for loc in locs {
        let last_ctxs = ctx_list;
        ctx_list = Vec::new();
        ctx_list.reserve(last_ctxs.len() * 2);
        for mut ctx in last_ctxs {
            if allow_skips {
                let mut newctx = ctx.clone();
                newctx.get_mut().skip(loc.id());
                // TODO: Check if this loc is required. If it is, we can't skip it.
                if can_win(world, newctx.get()) {
                    ctx_list.push(newctx);
                }
            }
            // Get the item and mark the location visited.
            ctx.visit(world, loc);
            ctx_list.push(ctx);
        }
    }

    if let Some((l, e)) = exit {
        let exit = world.get_exit(e);
        let loc = world.get_location(l);
        let last_ctxs = ctx_list;
        ctx_list = Vec::new();
        ctx_list.reserve(last_ctxs.len() * 2);
        for mut ctx in last_ctxs {
            if allow_skips {
                let mut newctx = ctx.clone();
                newctx.get_mut().skip(l);
                if can_win(world, newctx.get()) {
                    ctx_list.push(newctx);
                }
            }
            // Get the item and move along the exit.
            ctx.visit_exit(world, loc, exit);
            ctx_list.push(ctx);
        }
    }
    ctx_list
}

pub fn visit_simple<'a, W, T, L, E>(world: &'a W, ctx: &mut T) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context<'a> = T> + 'a,
    E: Exit<Context<'a> = T>,
{
    let mut ret = false;
    for (spot_id, spot_ctx) in access_simple(world, &ctx) {
        for act in world.get_spot_actions(spot_id) {
            if act.can_access(&spot_ctx) {
                act.perform(ctx);
            }
        }
        for loc in world.get_spot_locations(spot_id) {
            // If we can reach the spot with the location
            if ctx.todo(loc.id()) && loc.can_access(&spot_ctx) {
                ctx.collect(loc.item());
                for canon_loc_id in world.get_canon_locations(loc.id()) {
                    ctx.skip(canon_loc_id);
                }
                ctx.visit(loc.id());
                ret = true;
            }
        }
    }
    ret
}

pub fn can_win<'a, W, T, L, E>(world: &'a W, ctx: &T) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context<'a> = T> + 'a,
    E: Exit<Context<'a> = T>,
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
            return false;
        }
    }
    true
}
