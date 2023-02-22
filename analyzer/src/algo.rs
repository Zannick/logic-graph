#![allow(unused_variables)]

use crate::access::*;
use crate::context::*;
use crate::greedy::*;
use crate::heap::LimitedHeap;
use crate::world::*;
use std::fmt::Debug;

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

    let m = minimize_playthrough(world, startctx.get(), &wonctx);

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
    let mut attempts = 0;
    let mut wins = 0;
    while !heap.is_empty() {
        let ctx = heap.pop().unwrap();
        if world.won(ctx.get()) {
            println!(
                "Found winning path after {} attempts, in estimated {}ms, with {} remaining (of which {} are > {})",
                attempts,
                ctx.elapsed(),  heap.len(),
                heap.iter().filter(|c| c.elapsed() > ctx.elapsed()+10000).count(),
                ctx.elapsed()+10000
            );
            let m = minimize_playthrough(world, startctx.get(), &ctx);
            println!("Minimized it to {}ms", m.elapsed());
            println!("{}", m.history_str());

            if wins < 2 {
                wins += 1;
                heap.set_max_time(ctx.elapsed());
                heap.set_max_time(m.elapsed());
                println!("Max time to consider is now: {}ms", heap.max_time());
                continue;
            }
            return;
        }
        attempts += 1;
        if attempts % 10000 == 0 {
            println!(
                "Attempt {} (heap size {}): {}",
                attempts,
                heap.len(),
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
    if wins == 0 {
        println!("Did not find a winner after {} attempts", attempts);
    }
}
