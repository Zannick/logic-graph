#![allow(unused_variables)]

use crate::access::*;
use crate::context::*;
use crate::world::*;
use core::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Debug;

pub fn explore<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    heap: &mut BinaryHeap<Reverse<ContextWrapper<T>>>,
) where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    let spot_map = access(world, ctx);
    //println!("{:#?}", &spot_map);
    for (spot_id, mut spot_data) in spot_map {
        // Spot must have accessible locations with visited Status None
        if world
            .get_spot_locations(spot_id)
            .iter()
            .any(|loc| spot_data.get().todo(loc.id()) && loc.can_access(spot_data.get()))
        {
            spot_data.lastmode = Mode::Explore;
            heap.push(Reverse(spot_data));
        }
    }
}

pub fn visit_locations<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    heap: &mut BinaryHeap<Reverse<ContextWrapper<T>>>,
) where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    let mut exit = None;
    let locs: Vec<&L> = world
        .get_spot_locations(ctx.get().position())
        .iter()
        .filter(|loc| {
            if !ctx.get().todo(loc.id()) || !loc.can_access(ctx.get()) {
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

    let mut ctx_list = vec![ctx];
    for loc in locs {
        let last_ctxs = ctx_list;
        ctx_list = Vec::new();
        ctx_list.reserve(last_ctxs.len() * 2);
        for mut ctx in last_ctxs {
            let mut newctx = ctx.clone();
            newctx.get_mut().skip(loc.id());
            // TODO: Check if this loc is required. If it is, we can't skip it.
            ctx_list.push(newctx);
            // Get the item and mark the location visited.
            ctx.get_mut().visit(loc.id());
            ctx.get_mut().collect(loc.item());
            ctx.elapse(loc.time());
            ctx.history.push(History::Get(loc.item(), loc.id()));
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
            let mut newctx = ctx.clone();
            newctx.get_mut().skip(l);
            // TODO: Check if this loc is required. If it is, we can't skip it.
            ctx_list.push(newctx);
            // Get the item and move along the exit, recording both.
            ctx.get_mut().visit(l);
            ctx.get_mut().collect(loc.item());
            ctx.elapse(loc.time());
            ctx.get_mut().set_position(exit.dest());
            ctx.elapse(exit.time());
            ctx.history.push(History::Move(e));
            ctx.history.push(History::Get(loc.item(), l));
            ctx_list.push(ctx);
        }
    }

    heap.extend(ctx_list.into_iter().map(|mut c| {
        c.lastmode = Mode::Check;
        Reverse(c)
    }));
}

pub fn do_the_thing<W, T, L, E>(world: &W, ctx: T)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    // TODO: add 'time' property limit to heap wrapper to limit execution time
    // for benchmarking.
    // TODO: alter heap sort order to prioritize states with more items/less unvisited places/less time
    // instead of just less time. not just more items (that's DFS) but add some heuristic bonus
    // or penalize history length.
    let mut heap = BinaryHeap::new();
    let ctx = ContextWrapper::new(ctx);
    heap.push(Reverse(ctx));

    while !heap.is_empty() {
        let ctx = heap.pop().unwrap().0;
        match ctx.lastmode {
            Mode::None => {
                explore(world, ctx, &mut heap);
            }
            Mode::Explore => {
                visit_locations(world, ctx, &mut heap);
            }
            _ => println!("{}", ctx.info()),
        }
    }
}
