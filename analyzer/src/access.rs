use crate::context::*;
use crate::world::*;
use std::cmp::Reverse;
use std::collections::VecDeque;
use std::collections::{BinaryHeap, HashMap};

pub fn expand<W, T, E, Wp>(
    world: &W,
    ctx: &ContextWrapper<T>,
    dist_map: &HashMap<E::SpotId, ContextWrapper<T>>,
    spot_heap: &mut BinaryHeap<Reverse<ContextWrapper<T>>>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId>,
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
            insert(warp.dest(ctx.get()), warp.time(), History::Warp(warp.id()));
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
pub fn access<W, T, E>(world: &W, ctx: ContextWrapper<T>) -> HashMap<E::SpotId, ContextWrapper<T>>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId>,
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

    dist_map
}

/// Variant of `access` that does not write hist or time, ideal for beatability checks
pub fn access_simple<W, T, E>(world: &W, ctx: T) -> HashMap<<E as Exit>::SpotId, T>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId>,
{
    let pos = ctx.position();
    let mut spot_map = HashMap::new();
    let mut ctx_queue = VecDeque::new();
    spot_map.insert(pos, ctx);

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

/*
pub fn visit_fanout<W, T, E, L>(world: &W, ctx: ContextWrapper<T>) -> VecDeque<ContextWrapper<T>>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
{
    let pos = ctx.get().position();
    let mut ctx_queue = VecDeque::new();
    ctx_queue
}
*/