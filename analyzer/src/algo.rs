#![allow(unused_variables)]

use crate::access::*;
use crate::context::*;
use crate::greedy::*;
use crate::heap::LimitedHeap;
use crate::world::*;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;

pub fn explore<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    heap: &mut LimitedHeap<T>,
) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<ExitId = L::ExitId, Context = T, Currency = L::Currency>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = L::Currency>,
{
    let spot_map = accessible_spots(world, ctx);
    let mut vec = Vec::new();

    for (spot_id, mut spot_data) in spot_map.into_iter() {
        // Spot must have accessible locations with visited Status None
        if spot_has_locations(world, spot_data.get()) {
            vec.push(spot_data);
        } else if spot_has_actions(world, &spot_data) {
            spot_data.penalize(1000);
            vec.push(spot_data);
        }
    }
    if vec.is_empty() {
        return vec;
    }

    vec.sort_unstable_by_key(|el| el.elapsed());
    let shortest = vec[0].elapsed();
    // Suppose the distances to these spots are (delta from the first one) 0, 2, 3, 5, 10.
    // We want penalties to increase somewhat quadratically based on count (not just distance).
    // Penalties:
    // First el: 0. Second el: 0. Third el: 2nd-1st (2).
    // Fourth el: (2nd-1st)*2 + 3rd-2nd = 3rd+2nd - 2*1st, (4+1)
    // Fifth el: (3rd-1st)*2 + 4th-3rd = 4th+3rd - 2*1st, (6+2)
    // that's 0, 0, 2, 5, 8
    // penalties for 0, 1, 2, 3, 4, 5, 6: 0, 0, 1, 3, 7, 15, 31
    for i in 2..vec.len() {
        let penalty = vec[i].elapsed() + vec[i-1].elapsed() - 2 * shortest;
        vec[i].penalize(penalty);
    }
    vec
}

pub fn visit_locations<W, T, L, E>(world: &W, ctx: ContextWrapper<T>, heap: &mut LimitedHeap<T>)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T, Currency = L::Currency>,
{
    let mut ctx_list = vec![ctx];
    let (mut locs, exit) = visitable_locations(world, ctx_list[0].get());
    if locs.is_empty() && exit == None {
        return;
    }
    locs.sort_unstable_by_key(|loc| loc.time());
    for loc in locs {
        let last_ctxs = ctx_list;
        ctx_list = Vec::new();
        ctx_list.reserve(last_ctxs.len() * 2);
        for mut ctx in last_ctxs {
            if ctx.get().todo(loc.id()) && loc.can_access(ctx.get()) {
                // TODO: Add a better way to prevent this from causing too wide a branching factor
                // or remove.
                if !loc.is_free() {
                    let mut newctx = ctx.clone();
                    newctx.get_mut().skip(loc.id());
                    // Check if this loc is required. If it is, we can't skip it.
                    if can_win(world, newctx.get()) {
                        ctx_list.push(newctx);
                    }
                }

                // Get the item and mark the location visited.
                ctx.visit(world, loc);
            }
            ctx_list.push(ctx);
        }
    }

    if let Some((l, e)) = exit {
        let exit = world.get_exit(e);
        let loc = world.get_location(l);
        for ctx in ctx_list.iter_mut() {
            // Get the item and move along the exit.
            ctx.visit_exit(world, loc, exit);
        }
    }
    heap.extend(ctx_list);
}

pub fn activate_actions<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    penalty: i32,
    heap: &mut LimitedHeap<T>,
) where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    for act in world
        .get_global_actions()
        .iter()
        .chain(world.get_spot_actions(ctx.get().position()))
    {
        if act.can_access(ctx.get()) && ctx.is_useful(act) {
            let mut c2 = ctx.clone();
            c2.activate(act);
            if can_win(world, c2.get()) {
                c2.penalize(penalty);
                heap.push(c2);
            }
        }
    }
}

fn search_step<W, T, L, E>(world: &W, ctx: ContextWrapper<T>, heap: &mut LimitedHeap<T>)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId, LocId = E::LocId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T, Currency = L::Currency>,
{
    // The process will look more like this:
    // 1. explore -> vec of spot ctxs with penalties applied
    // 2. get largest dist
    // 3. (activate_actions) for each ctx, check for global actions and spot actions
    // 4. (visit_locations) for each ctx, get all available locations
    let spot_ctxs = explore(world, ctx, heap);
    if let (Some(s), Some(f)) = (spot_ctxs.first(), spot_ctxs.last()) {
        let max_dist = f.elapsed() - s.elapsed();
        for ctx in spot_ctxs.iter() {
            activate_actions(world, ctx.clone(), max_dist, heap);
        }
        for ctx in spot_ctxs {
            visit_locations(world, ctx, heap);
        }
    }
}

pub fn minimize_nongreedy<W, T, L, E>(
    world: &W,
    startctx: &T,
    wonctx: &ContextWrapper<T>,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId, LocId = E::LocId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T, Currency = L::Currency>,
{
    find_one(world, minimize(world, startctx, wonctx), wonctx.elapsed())
}

pub fn find_one<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_time: i32,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId, LocId = E::LocId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T, Currency = L::Currency>,
{
    if !can_win(world, ctx.get()) {
        panic!("Trying to solve a minimized search that can't win");
    }
    let mut heap = LimitedHeap::new();
    heap.set_max_time(max_time + 1);
    heap.push(ctx);
    let mut iters = 0;
    while let Some(ctx) = heap.pop() {
        if world.won(ctx.get()) {
            return Some(ctx);
        }
        iters += 1;
        search_step(world, ctx, &mut heap);
    }
    println!("Failed to find minimized win after {} mini-rounds", iters);
    None
}

pub fn search<W, T, L, E>(world: &W, mut ctx: T)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId, LocId = E::LocId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T, Currency = L::Currency>,
{
    world.skip_unused_items(&mut ctx);
    let startctx = ContextWrapper::new(ctx);
    let wonctx = greedy_search(world, &startctx).expect("Did not find a solution");

    let m = minimize_greedy(world, startctx.get(), &wonctx);

    println!(
        "Found greedy solution of {}ms, minimized to {}ms",
        wonctx.elapsed(),
        m.elapsed()
    );

    let mut heap = LimitedHeap::new();
    heap.set_lenient_max_time(wonctx.elapsed());
    heap.set_lenient_max_time(m.elapsed());
    heap.push(startctx.clone());
    println!("Max time to consider is now: {}ms", heap.max_time());
    let mut iters = 0;
    let mut winner = None;
    while let Some(ctx) = heap.pop() {
        if world.won(ctx.get()) {
            println!(
                "Found winning path after {} rounds, in estimated {}ms, with {} remaining in heap",
                iters,
                ctx.elapsed(),
                heap.len()
            );
            heap.set_lenient_max_time(ctx.elapsed());
            if let Some(m) = minimize_nongreedy(world, startctx.get(), &ctx) {
                heap.set_lenient_max_time(m.elapsed());
                println!("Minimized it to {}ms", m.elapsed());
                if m.elapsed() > ctx.elapsed() {
                    println!("Weird, it got slower?");
                    let mut orig = File::create("/tmp/orig").unwrap();
                    orig.write(ctx.history_str().as_bytes()).unwrap();
                    let mut min = File::create("/tmp/new").unwrap();
                    min.write(m.history_str().as_bytes()).unwrap();
                    return;
                }
                winner = Some(m);
            } else {
                winner = Some(ctx);
            }

            println!("Max time to consider is now: {}ms", heap.max_time());
            continue;
        }
        iters += 1;
        if iters % 10000 == 0 {
            let (iskips, pskips) = heap.stats();
            println!(
                "Round {} (heap size {}, skipped {} pushes + {} pops):\n  {}",
                iters,
                heap.len(),
                iskips,
                pskips,
                ctx.info()
            );
        }
        search_step(world, ctx, &mut heap);
    }
    let (iskips, pskips) = heap.stats();
    println!(
        "Finished after {} rounds, skipped {} pushes + {} pops",
        iters, iskips, pskips
    );
    if let Some(m) = winner {
        println!("Final result: est. {}ms\n{}", m.elapsed(), m.history_str());
    } else {
        println!("Did not find a winner");
    }
}
