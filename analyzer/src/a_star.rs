use crate::context::*;
use crate::heap::HeapElement;
use crate::world::*;
use crate::CommonHasher;
use sort_by_derive::SortBy;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

pub trait SortableCtxWrapper<T: Ctx, P: Ord>: std::fmt::Debug + Ord {
    fn ctx(&self) -> &ContextWrapper<T>;
    fn copy_update(&self, newctx: ContextWrapper<T>, score: P) -> Self;
    fn should_keep(&self, _max_depth: i8) -> bool {
        true
    }
    fn can_continue(&self, _max_depth: i8) -> bool {
        true
    }
}

impl<T: Ctx> SortableCtxWrapper<T, u32> for HeapElement<T> {
    fn ctx(&self) -> &ContextWrapper<T> {
        &self.el
    }
    fn copy_update(&self, newctx: ContextWrapper<T>, score: u32) -> Self {
        HeapElement { score, el: newctx }
    }
}

#[derive(Debug, SortBy)]
pub struct ScoredCtxWithActionCounter<T: Ctx> {
    #[sort_by]
    pub(crate) score: u32,
    pub(crate) el: ContextWrapper<T>,
    pub(crate) counter: i8,
}

impl<T: Ctx> SortableCtxWrapper<T, u32> for ScoredCtxWithActionCounter<T> {
    fn ctx(&self) -> &ContextWrapper<T> {
        &self.el
    }
    fn copy_update(&self, newctx: ContextWrapper<T>, score: u32) -> Self {
        let counter = if matches!(newctx.recent_history().last(), Some(History::A(..))) {
            self.counter + 1
        } else {
            self.counter
        };
        ScoredCtxWithActionCounter {
            score,
            el: newctx,
            counter,
        }
    }
    fn should_keep(&self, max_depth: i8) -> bool {
        self.counter <= max_depth
    }
    fn can_continue(&self, max_depth: i8) -> bool {
        self.counter < max_depth
    }
}

pub fn expand_exits_astar<W, T, E, H>(
    world: &W,
    el: &H,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<H>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<u32>,
) where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    H: SortableCtxWrapper<T, u32>,
{
    let ctx = el.ctx();
    for exit in world.get_spot_exits(ctx.get().position()) {
        if exit.can_access(ctx.get(), world) {
            let mut newctx = ctx.clone();
            newctx.exit(world, exit);
            let elapsed = newctx.elapsed();
            if !states_seen.contains(newctx.get()) && elapsed <= max_time {
                if let Some(score) = score_func(&newctx) {
                    spot_heap.push(Reverse(el.copy_update(newctx, score)));
                }
            }
        }
    }
}

pub fn expand_actions_astar<W, T, E, H>(
    world: &W,
    el: &H,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<H>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<u32>,
) where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    H: SortableCtxWrapper<T, u32>,
{
    let ctx = el.ctx();
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
                    let new_el = el.copy_update(newctx, score);
                    spot_heap.push(Reverse(new_el));
                }
            }
        }
    }
}

// This is mainly for move_to.
pub fn expand_local_astar<W, T, E, Wp, H>(
    world: &W,
    el: &H,
    movement_state: T::MovementState,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<H>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<u32>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
    H: SortableCtxWrapper<T, u32>,
{
    let ctx = el.ctx();
    for &dest in world.get_area_spots(ctx.get().position()) {
        let ltt = ctx.get().local_travel_time(movement_state, dest);
        if ltt < u32::MAX {
            let mut newctx = ctx.clone();
            newctx.move_local(world, dest, ltt);
            let elapsed = newctx.elapsed();
            if !states_seen.contains(newctx.get()) && elapsed <= max_time {
                if let Some(score) = score_func(&newctx) {
                    spot_heap.push(Reverse(el.copy_update(newctx, score)));
                } else {
                    log::warn!("Moved locally to {} but got no score; disconnected?", dest);
                }
            }
        }
    }
}

pub fn expand_astar<W, T, E, Wp, H>(
    world: &W,
    el: &H,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<H>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<u32>,
    allow_local: bool,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
    H: SortableCtxWrapper<T, u32>,
{
    let ctx = el.ctx();
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
                        spot_heap.push(Reverse(el.copy_update(newctx, score)));
                    } else {
                        log::warn!("Followed CE to {} but got no score; disconnected?", ce.dst);
                    }
                }
            }
        }
        if allow_local {
            expand_local_astar(
                world,
                el,
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
            el,
            movement_state,
            states_seen,
            max_time,
            spot_heap,
            score_func,
        );
    }

    expand_exits_astar(world, el, states_seen, max_time, spot_heap, score_func);

    for warp in world.get_warps() {
        if warp.can_access(ctx.get(), world) {
            let mut newctx = ctx.clone();
            newctx.warp(world, warp);
            let elapsed = newctx.elapsed();
            if !states_seen.contains(newctx.get()) && elapsed <= max_time {
                if let Some(score) = score_func(&newctx) {
                    spot_heap.push(Reverse(el.copy_update(newctx, score)));
                }
            }
        }
    }
}
