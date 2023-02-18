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
) -> ()
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    let spot_map = access(world, ctx);
    println!("{:#?}", &spot_map);
    //let new_spots = vec![];
    for (spot_id, spot_data) in spot_map {
        // 1. Spot must have accessible locations with visited Status None
        if !world
            .get_spot_locations(spot_id)
            .iter()
            .any(|loc| spot_data.get().todo(loc.id()) && loc.can_access(spot_data.get()))
        {
            continue;
        }
        // 2. Add dist and path to the current ctx and add to the heap
        //newctx.set
    }
}

pub fn do_the_thing<W, T, L, E>(world: &W, ctx: T)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    let mut heap = BinaryHeap::new();
    let ctx = ContextWrapper::new(ctx);
    heap.push(Reverse(ctx));

    while !heap.is_empty() {
        let ctx = heap.pop().unwrap().0;
        match ctx.lastmode {
            Mode::None => {
                let res = explore(world, ctx, &mut heap);
            }
            _ => (),
        }
    }
}
