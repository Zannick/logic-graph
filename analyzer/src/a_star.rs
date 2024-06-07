use crate::context::*;
use crate::heap::HeapElement;
use crate::world::*;
use crate::CommonHasher;
use sort_by_derive::SortBy;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

pub trait SortableCtxWrapper<T: Ctx, P: Ord>: Ord {
    fn ctx(&self) -> &ContextWrapper<T>;
    fn copy_update(&self, newctx: ContextWrapper<T>, score: P) -> Self;
    fn new_incr(&self, newctx: ContextWrapper<T>, score: P) -> Self;
    fn can_continue(&self, _max_depth: usize) -> bool {
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
    fn new_incr(&self, newctx: ContextWrapper<T>, score: u32) -> Self {
        HeapElement { score, el: newctx }
    }
}

#[derive(Debug, SortBy)]
pub struct ScoredCtxWithActionCounter<T: Ctx, P: Ord + std::hash::Hash> {
    #[sort_by]
    pub(crate) score: P,
    pub(crate) el: ContextWrapper<T>,
    pub(crate) counter: usize,
}

impl<T, P> SortableCtxWrapper<T, P> for ScoredCtxWithActionCounter<T, P>
where
    T: Ctx,
    P: Ord + std::hash::Hash,
{
    fn ctx(&self) -> &ContextWrapper<T> {
        &self.el
    }
    fn copy_update(&self, newctx: ContextWrapper<T>, score: P) -> Self {
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
    fn new_incr(&self, newctx: ContextWrapper<T>, score: P) -> Self {
        ScoredCtxWithActionCounter {
            score,
            el: newctx,
            counter: self.counter + 1,
        }
    }
    fn can_continue(&self, max_depth: usize) -> bool {
        self.counter < max_depth
    }
}

pub fn expand_exits_astar<W, T, E, H, P>(
    world: &W,
    el: &H,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<H>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<P>,
) where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    H: SortableCtxWrapper<T, P>,
    P: Ord + std::hash::Hash,
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

pub fn expand_actions_astar<W, T, E, H, P>(
    world: &W,
    el: &H,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<H>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<P>,
) where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    H: SortableCtxWrapper<T, P>,
    P: Ord + std::hash::Hash,
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
                    spot_heap.push(Reverse(el.new_incr(newctx, score)));
                }
            }
        }
    }
}

// This is mainly for move_to.
pub fn expand_local_astar<W, T, E, Wp, H, P>(
    world: &W,
    el: &H,
    movement_state: T::MovementState,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<H>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<P>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
    H: SortableCtxWrapper<T, P>,
    P: Ord + std::hash::Hash,
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
                }
            }
        }
    }
}

pub fn expand_astar<W, T, E, Wp, H, P>(
    world: &W,
    el: &H,
    states_seen: &HashSet<T, CommonHasher>,
    max_time: u32,
    spot_heap: &mut BinaryHeap<Reverse<H>>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<P>,
    allow_local: bool,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Location: Location<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
    H: SortableCtxWrapper<T, P>,
    P: Ord + std::hash::Hash,
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
