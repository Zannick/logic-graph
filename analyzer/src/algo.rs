#![allow(unused_variables)]

use crate::access::*;
use crate::context::*;
use crate::world::*;
use std::collections::BinaryHeap;
use std::fmt::Debug;

pub fn explore<T, S, L, E>(
    world: &impl World<
        Context = T,
        SpotId = S,
        Exit = impl Exit<ExitId = E, SpotId = S> + Accessible<Context = T>,
        Location = impl Location<LocId = L> + Accessible<Context = T>,
    >,
    ctx: &ContextWrapper<T>,
    heap: &mut BinaryHeap<ContextWrapper<T>>,
) -> ()
where
    T: Ctx<SpotId = S, LocationId = L, ExitId = E> + Debug,
    S: Id,
    L: Id,
    E: Id,
{
    let spot_map = access(world, &ctx);
    //let new_spots = vec![];
    for (spot_id, dist) in spot_map {
        // 1. Spot must have accessible locations with visited Status None
        if !world
            .get_spot_locations(spot_id)
            .iter()
            .any(|loc| ctx.get().todo(loc.id()) && loc.can_access(ctx.get()))
        {
            continue;
        }
    }
}

pub fn do_the_thing<T, S, L, E>(
    world: &impl World<
        Context = T,
        SpotId = S,
        Exit = impl Exit<ExitId = E, SpotId = S> + Accessible<Context = T>,
        Location = impl Location<LocId = L> + Accessible<Context = T>,
    >,
    ctx: T,
) where
    T: Ctx<SpotId = S, LocationId = L, ExitId = E> + Debug,
    S: Id,
    L: Id,
    E: Id,
{
    let mut heap = BinaryHeap::new();
    let ctx = ContextWrapper::new(ctx);
    heap.push(ctx);

    while !heap.is_empty() {
        let ctx = heap.pop().unwrap();
        match ctx.lastmode {
            Mode::None => {
                let res = explore(world, &ctx, &mut heap);
            }
            _ => (),
        }
    }
}
