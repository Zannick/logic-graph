use crate::context::*;
use crate::world::*;
use core::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Debug)]
pub struct SpotAccess<T>
where
    T: Id,
{
    pub dist: i32,
    pub id: T,
}

pub fn expand<T, S>(
    world: &impl World<Context = T, SpotId = S>,
    ctx: &T,
    start: S,
    base_time: i32,
    dist_map: &HashMap<S, i32>,
    spot_heap: &mut BinaryHeap<Reverse<SpotAccess<S>>>,
) where
    T: Ctx<SpotId = S>,
    S: Id,
{
    for spot in world.get_area_spots(start) {
        if !dist_map.contains_key(spot) {
            println!("Adding area spot via {:?} ==> {:?}", start, spot);
            let local = ctx.local_travel_time(start, *spot);
            if local < 0 {
                panic!(
                    "Could not traverse within area: start={:?} dest={:?}",
                    start, spot
                );
            }
            spot_heap.push(Reverse(SpotAccess {
                id: *spot,
                // if we use ctx.position for allowed movements we need to set it
                dist: base_time + ctx.local_travel_time(start, *spot),
            }));
        }
    }

    for exit in world.get_spot_exits(start) {
        // TODO: do we need to update ctx.position for can_access rules?
        println!(
            "Checking {:?} ==> {:?} access: {:?}",
            start,
            exit.dest(),
            exit.can_access(ctx)
        );
        if !dist_map.contains_key(exit.dest()) && exit.can_access(ctx) {
            spot_heap.push(Reverse(SpotAccess {
                id: *exit.dest(),
                dist: base_time + exit.time(),
            }));
        }
    }
}

pub fn access<T, S>(
    world: &impl World<Context = T, SpotId = S>,
    ctx: &ContextWrapper<T>,
) -> HashMap<S, i32>
where
    T: Ctx<SpotId = S>,
    S: Id,
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
    println!("{:#?}", spot_heap);
    while !spot_heap.is_empty() {
        let spot_found = spot_heap.pop().expect("nonempty").0;
        if !dist_map.contains_key(&spot_found.id) {
            dist_map.insert(spot_found.id, spot_found.dist);
            let mut c = ctx.get().clone();
            c.set_position(spot_found.id);
            expand(
                world,
                &c,
                spot_found.id,
                spot_found.dist,
                &dist_map,
                &mut spot_heap,
            );
        }
    }

    dist_map
}
