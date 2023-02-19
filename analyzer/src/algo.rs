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
    let mut tmp_heap = BinaryHeap::new();
    //println!("{:#?}", &spot_map);
    for (spot_id, mut spot_data) in spot_map {
        // Spot must have accessible locations with visited Status None
        if world
            .get_spot_locations(spot_id)
            .iter()
            .any(|loc| spot_data.get().todo(loc.id()) && loc.can_access(spot_data.get()))
        {
            spot_data.lastmode = Mode::Explore;
            tmp_heap.push(Reverse(spot_data));
        } else if world
            .get_spot_actions(spot_id)
            .iter()
            .any(|act| act.can_access(spot_data.get()))
        {
            let mut actdata = spot_data.clone();
            actdata.elapse(1000);
            tmp_heap.push(Reverse(actdata));
        }
        if tmp_heap.len() > 3 {
            break;
        }
    }
    heap.append(&mut tmp_heap);
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
    heap.extend(visit_fanout(world, ctx, false).into_iter().map(|mut c| {
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
    let mut attempts = 0;

    while !heap.is_empty() {
        let ctx = heap.pop().unwrap().0;
        if world.won(ctx.get()) {
            println!(
                "Found winning path after {} attempts, in estimated {}ms, with {} remaining (of which {} are > {})",
                attempts,
                ctx.elapsed(),  heap.len(),
                heap.iter().filter(|c| c.0.elapsed() > ctx.elapsed()+10000).count(),
                ctx.elapsed()+10000
            );
            println!("{:?}", ctx.history);

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
    println!("Did not find a winner after {} attempts", attempts);
}
