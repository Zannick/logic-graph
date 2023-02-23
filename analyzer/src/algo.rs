#![allow(unused_variables)]

use crate::access::*;
use crate::context::*;
use crate::greedy::*;
use crate::heap::LimitedHeap;
use crate::world::*;
use std::fmt::Debug;
use std::fs::File;
use std::io::Write;

pub fn explore<W, T, L, E>(world: &W, ctx: ContextWrapper<T>, heap: &mut LimitedHeap<T>)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    let spot_map = access(world, ctx);
    let mut vec = Vec::new();
    //println!("{:#?}", &spot_map);
    for (spot_id, mut spot_data) in spot_map {
        // Spot must have accessible locations with visited Status None
        if spot_has_locations(world, spot_data.get(), spot_id) {
            spot_data.lastmode = Mode::Explore;
            vec.push(spot_data);
        } else if spot_has_actions(world, spot_data.get(), spot_id) {
            let mut actdata = spot_data.clone();
            actdata.lastmode = Mode::Explore;
            actdata.penalize(1000);
            vec.push(actdata);
        }
    }
    if vec.is_empty() {
        return;
    }

    vec.sort_unstable_by_key(|el| el.elapsed());
    let mut penalty = 0;
    let mut last: i32 = 0;
    // Suppose the distances to these spots are (delta from the first one) 0, 2, 3, 5, 10.
    // We want penalties to increase somewhat quadratically based on count (not just distance).
    // Penalties:
    // First el: 0. Second el: 0. Third el: 2nd-1st (2).
    // Fourth el: (2nd-1st)*2 + 3rd-2nd, aka twice #3's penalty + diff (4+1)
    // Fifth el: twice the last penalty + diff (10 + 10)
    // that's 0, 0, 2, 5, 10
    // penalties for 0, 1, 2, 3, 4, 5, 6: 0, 0, 1, 3, 7, 15, 31
    for mut el in vec.into_iter() {
        if last > 0 {
            el.penalize(penalty);
            penalty += penalty + el.elapsed() - last;
        }
        last = el.elapsed();
        heap.push(el);
    }
}

pub fn visit_locations<W, T, L, E>(world: &W, ctx: ContextWrapper<T>, heap: &mut LimitedHeap<T>)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    heap.extend(visit_fanout(world, ctx, false).into_iter().map(|mut c| {
        c.lastmode = Mode::Check;
        c
    }));
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
    E: Exit + Accessible<Context = T>,
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
    E: Exit + Accessible<Context = T>,
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
            println!("Minimized to {}ms", ctx.elapsed());
            return Some(ctx);
        }
        iters += 1;
        match ctx.lastmode {
            Mode::None | Mode::Check => {
                explore(world, ctx, &mut heap);
            }
            Mode::Explore => {
                visit_locations(world, ctx, &mut heap);
            }
            _ => (),
        }
    }
    println!("Failed to find minimized win after {} mini-rounds", iters);
    None
}

pub fn search<W, T, L, E>(world: &W, mut ctx: T)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId, LocId = E::LocId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
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
    heap.set_max_time(wonctx.elapsed());
    heap.set_max_time(m.elapsed());
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
            heap.set_max_time(ctx.elapsed());
            if let Some(m) = minimize_nongreedy(world, startctx.get(), &ctx) {
                heap.set_max_time(m.elapsed());
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
        match ctx.lastmode {
            Mode::None | Mode::Check => {
                explore(world, ctx, &mut heap);
            }
            Mode::Explore => {
                visit_locations(world, ctx, &mut heap);
            }
            _ => println!("{}", ctx.info()),
        }
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
