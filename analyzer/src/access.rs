use crate::context::*;
use crate::world::*;
use std::cmp::Reverse;
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
            // We're copying the whole context on every step, which is probably
            // super inefficient; we only really copy because position may be relevant
            // for connection checks. If we tracked the "context" state separately
            // from the item state, it might be less copying.
            let mut newctx = ctx.clone();
            newctx.get_mut().set_position(*spot);
            newctx.history.push(History::MoveLocal(*spot));
            newctx.elapse(local);
            spot_heap.push(Reverse(newctx));
        }
    }

    for exit in world.get_spot_exits(ctx.get().position()) {
        if !dist_map.contains_key(&exit.dest()) && exit.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.get_mut().set_position(exit.dest());
            newctx.history.push(History::Move(exit.id()));
            newctx.elapse(exit.time());
            spot_heap.push(Reverse(newctx));
        }
    }

    for warp in world.get_warps() {
        if !dist_map.contains_key(&warp.dest(ctx.get())) && warp.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.get_mut().set_position(warp.dest(ctx.get()));
            newctx.history.push(History::Warp(warp.dest(ctx.get())));
            newctx.elapse(warp.time());
            spot_heap.push(Reverse(newctx));
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
