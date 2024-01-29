use crate::context::*;
use crate::heap::HeapElement;
use crate::world::*;
use crate::CommonHasher;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

pub fn expand_exits_astar<W, T, E>(
    world: &W,
    ctx: &ContextWrapper<T>,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<HeapElement<T>>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<u32>,
) where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
{
    for exit in world.get_spot_exits(ctx.get().position()) {
        if exit.can_access(ctx.get(), world) {
            let mut newctx = ctx.clone();
            newctx.exit(world, exit);
            let elapsed = newctx.elapsed();
            if !states_seen.contains(newctx.get()) && elapsed <= max_time {
                if let Some(score) = score_func(&newctx) {
                    spot_heap.push(Reverse(HeapElement { score, el: newctx }));
                }
            }
        }
    }
}

pub fn expand_actions_astar<W, T, E>(
    world: &W,
    ctx: &ContextWrapper<T>,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<HeapElement<T>>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<u32>,
) where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
{
    for act in world
        .get_global_actions()
        .iter()
        .chain(world.get_spot_actions(ctx.get().position()))
    {
        if act.can_access(ctx.get(), world) {
            let mut newctx = ctx.clone();
            newctx.activate(world, act);
            let elapsed = newctx.elapsed();
            if !states_seen.contains(newctx.get()) && elapsed <= max_time {
                if let Some(score) = score_func(&newctx) {
                    spot_heap.push(Reverse(HeapElement { score, el: newctx }));
                }
            }
        }
    }
}

// This is mainly for move_to.
pub fn expand_local_astar<W, T, E, Wp>(
    world: &W,
    ctx: &ContextWrapper<T>,
    movement_state: T::MovementState,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<HeapElement<T>>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<u32>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    for &dest in world.get_area_spots(ctx.get().position()) {
        let ltt = ctx.get().local_travel_time(movement_state, dest);
        if ltt < u32::MAX {
            let mut newctx = ctx.clone();
            newctx.move_local(world, dest, ltt);
            let elapsed = newctx.elapsed();
            if !states_seen.contains(newctx.get()) && elapsed <= max_time {
                if let Some(score) = score_func(&newctx) {
                    spot_heap.push(Reverse(HeapElement { score, el: newctx }));
                } else {
                    log::warn!("Moved locally to {} but got no score; disconnected?", dest);
                }
            }
        }
    }
}

pub fn expand_astar<W, T, E, Wp>(
    world: &W,
    ctx: &ContextWrapper<T>,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<HeapElement<T>>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<u32>,
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
            if ce.can_access(world, ctx.get(), movement_state) {
                let mut newctx = ctx.clone();
                newctx.move_condensed_edge(world, ce);
                let elapsed = newctx.elapsed();
                if !states_seen.contains(newctx.get()) && elapsed <= max_time {
                    if let Some(score) = score_func(&newctx) {
                        spot_heap.push(Reverse(HeapElement { score, el: newctx }));
                    } else {
                        log::warn!("Followed CE to {} but got no score; disconnected?", ce.dst);
                    }
                }
            }
        }
        if allow_local {
            expand_local_astar(
                world,
                ctx,
                movement_state,
                states_seen,
                max_time,
                spot_heap,
                score_func,
            );
        }
    } else {
        expand_local_astar(
            world,
            ctx,
            movement_state,
            states_seen,
            max_time,
            spot_heap,
            score_func,
        );
    }

    expand_exits_astar(world, ctx, states_seen, max_time, spot_heap, score_func);

    for warp in world.get_warps() {
        if warp.can_access(ctx.get(), world) {
            let mut newctx = ctx.clone();
            newctx.warp(world, warp);
            let elapsed = newctx.elapsed();
            if !states_seen.contains(newctx.get()) && elapsed <= max_time {
                if let Some(score) = score_func(&newctx) {
                    spot_heap.push(Reverse(HeapElement { score, el: newctx }));
                }
            }
        }
    }
}
