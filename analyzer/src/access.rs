use crate::context::*;
use crate::world::*;
use core::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct SpotAccess<T>
where
    T: Ctx,
{
    pub dist: i32,
    pub id: <T as Ctx>::SpotId,
}

pub fn expand<T>(
    world: &impl World<Context = T>,
    ctx: &T,
    start: <T as Ctx>::SpotId,
    base_time: i32,
    dist_map: &HashMap<<T as Ctx>::SpotId, i32>,
    spot_heap: &mut BinaryHeap<SpotAccess<T>>,
) where
    T: Ctx,
{
    for spot in world.get_area_spots(start) {
        if !dist_map.contains_key(spot) {
            spot_heap.push(Reverse(SpotAccess {
                id: spot,
                dist: base_time + ctx.local_travel_time_to(spot),
            }));
        }
    }

    for exit in world.get_spot_exits(start) {
        // TODO: do we need to update ctx.position for can_access rules?
        if !dist_map.contains_key(exit.dest()) && exit.can_access(ctx) {
            spot_heap.push(Reverse(SpotAccess {
                id: exit.dest(),
                dist: base_time + exit.time(),
            }));
        }
    }
}

pub fn access<T>(world: &impl World<Context = T>, ctx: &ContextWrapper<T>)
where
    T: Ctx,
{
    let mut spot_heap = BinaryHeap::new();
    let mut dist_map = HashMap::new();

    dist_map.insert(ctx.get().position(), 0);

    expand(
        world,
        ctx.get(),
        ctx.get().position(),
        0,
        &dist_map,
        &mut spot_heap,
    );
    while !spot_heap.is_empty() {
        let spot_found = spot_heap.pop().expect("nonempty").0;
        if !dist_map.contains_key(spot_found.id) {
            dist_map.insert(spot_found.id, spot_found.dist);
            expand(
                world,
                ctx.get(),
                spot_found.id,
                spot_found.dist,
                &dist_map,
                &mut spot_heap,
            );
        }
    }
}
