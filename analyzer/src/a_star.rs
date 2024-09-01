use crate::context::*;
use crate::priority::LimitedPriorityQueue;
use crate::world::*;
use crate::CommonHasher;

pub trait CtxWrapper<T: Ctx>: Clone + Eq + std::hash::Hash {
    fn ctx(&self) -> &ContextWrapper<T>;
    fn copy_update(&self, newctx: ContextWrapper<T>) -> Self;
    fn new_incr(&self, newctx: ContextWrapper<T>) -> Self;
    fn can_continue(&self, _max_depth: usize) -> bool {
        true
    }
}

impl<T: Ctx> CtxWrapper<T> for ContextWrapper<T> {
    fn ctx(&self) -> &ContextWrapper<T> {
        self
    }
    fn copy_update(&self, newctx: ContextWrapper<T>) -> Self {
        newctx
    }
    fn new_incr(&self, newctx: ContextWrapper<T>) -> Self {
        newctx
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CtxWithActionCounter<T: Ctx> {
    pub(crate) el: ContextWrapper<T>,
    pub(crate) counter: usize,
}

impl<T> CtxWrapper<T> for CtxWithActionCounter<T>
where
    T: Ctx,
{
    fn ctx(&self) -> &ContextWrapper<T> {
        &self.el
    }
    fn copy_update(&self, newctx: ContextWrapper<T>) -> Self {
        let counter = if matches!(newctx.recent_history().last(), Some(History::A(..))) {
            self.counter + 1
        } else {
            self.counter
        };
        CtxWithActionCounter {
            el: newctx,
            counter,
        }
    }
    fn new_incr(&self, newctx: ContextWrapper<T>) -> Self {
        CtxWithActionCounter {
            el: newctx,
            counter: self.counter + 1,
        }
    }
    fn can_continue(&self, max_depth: usize) -> bool {
        self.counter < max_depth
    }
}

pub fn expand_exits_astar<W, T, H, P>(
    world: &W,
    el: &H,
    max_time: u32,
    spot_heap: &mut LimitedPriorityQueue<H, T, P, CommonHasher>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<P>,
) where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
    H: CtxWrapper<T>,
    P: Clone + std::fmt::Debug + Ord + std::hash::Hash,
{
    let ctx = el.ctx();
    for exit in world.get_spot_exits(ctx.get().position()) {
        if exit.can_access(ctx.get(), world) {
            let mut newctx = ctx.clone();
            newctx.exit(world, exit);
            let elapsed = newctx.elapsed();
            if elapsed <= max_time {
                if let Some(score) = score_func(&newctx) {
                    let unique_key = newctx.get().clone();
                    spot_heap.push(el.copy_update(newctx), unique_key, score);
                }
            }
        }
    }
}

pub fn expand_actions_astar<W, T, H, P>(
    world: &W,
    el: &H,
    max_time: u32,
    spot_heap: &mut LimitedPriorityQueue<H, T, P, CommonHasher>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<P>,
) where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
    H: CtxWrapper<T>,
    P: Clone + Ord + std::hash::Hash,
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
            if elapsed <= max_time {
                if let Some(score) = score_func(&newctx) {
                    let unique_key = newctx.get().clone();
                    spot_heap.push(el.new_incr(newctx), unique_key, score);
                }
            }
        }
    }
}

// This is mainly for move_to.
pub fn expand_local_astar<W, T, H, P>(
    world: &W,
    el: &H,
    movement_state: T::MovementState,
    max_time: u32,
    spot_heap: &mut LimitedPriorityQueue<H, T, P, CommonHasher>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<P>,
) where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
    H: CtxWrapper<T>,
    P: Clone + std::fmt::Debug + Ord + std::hash::Hash,
{
    let ctx = el.ctx();
    for &dest in world.get_area_spots(ctx.get().position()) {
        let ltt = ctx.get().local_travel_time(movement_state, dest);
        if ltt < u32::MAX {
            let mut newctx = ctx.clone();
            newctx.move_local(world, dest, ltt);
            let elapsed = newctx.elapsed();
            if elapsed <= max_time {
                if let Some(score) = score_func(&newctx) {
                    let unique_key = newctx.get().clone();
                    spot_heap.push(el.copy_update(newctx), unique_key, score);
                }
            }
        }
    }
}

pub fn expand_astar<W, T, H, P>(
    world: &W,
    el: &H,
    max_time: u32,
    spot_heap: &mut LimitedPriorityQueue<H, T, P, CommonHasher>,
    score_func: &impl Fn(&ContextWrapper<T>) -> Option<P>,
    allow_local: bool,
) where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
    H: CtxWrapper<T>,
    P: Clone + std::fmt::Debug + Ord + std::hash::Hash,
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
                if elapsed <= max_time {
                    if let Some(score) = score_func(&newctx) {
                        let unique_key = newctx.get().clone();
                        spot_heap.push(el.copy_update(newctx), unique_key, score);
                    }
                }
            }
        }
        if allow_local {
            expand_local_astar(world, el, movement_state, max_time, spot_heap, score_func);
        }
    } else {
        expand_local_astar(world, el, movement_state, max_time, spot_heap, score_func);
    }

    expand_exits_astar(world, el, max_time, spot_heap, score_func);

    for warp in world.get_warps() {
        if warp.can_access(ctx.get(), world) {
            let mut newctx = ctx.clone();
            newctx.warp(world, warp);
            let elapsed = newctx.elapsed();
            if elapsed <= max_time {
                if let Some(score) = score_func(&newctx) {
                    let unique_key = newctx.get().clone();
                    spot_heap.push(el.copy_update(newctx), unique_key, score);
                }
            }
        }
    }
}
