use crate::context::*;
use crate::world::*;
use sort_by_derive::SortBy;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::fmt::Debug;

#[derive(Clone, Debug, SortBy)]
pub struct SpotAccess<T, S>
where
    T: Ctx,
    <T::World as World>::Exit: Exit<SpotId = S>,
    S: Id,
{
    #[sort_by]
    pub dist: i32,
    pub id: S,
    // this could store info about the path to the spot for better history
    pub path: Vec<History<T>>,
}

pub fn expand<W, T, E>(
    world: &W,
    ctx: &T,
    start: SpotAccess<T, E::SpotId>,
    dist_map: &HashMap<E::SpotId, i32>,
    spot_heap: &mut BinaryHeap<Reverse<SpotAccess<T, E::SpotId>>>,
) where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit + Accessible<Context = T>,
{
    for spot in world.get_area_spots(start.id) {
        if !dist_map.contains_key(spot) {
            let local = ctx.local_travel_time(start.id, *spot);
            if local < 0 {
                panic!(
                    "Could not travel within area: start={:?} dest={:?}",
                    start.id, spot
                );
            }
            let mut path = start.path.clone();
            path.push(History::MoveLocal(*spot));
            spot_heap.push(Reverse(SpotAccess {
                id: *spot,
                // if we use ctx.position for allowed movements we need to set it
                dist: start.dist + ctx.local_travel_time(start.id, *spot),
                path: path,
            }));
        }
    }

    for exit in world.get_spot_exits(start.id) {
        if !dist_map.contains_key(&exit.dest()) && exit.can_access(ctx) {
            let mut path = start.path.clone();
            path.push(History::Move(exit.id()));
            spot_heap.push(Reverse(SpotAccess {
                id: exit.dest(),
                dist: start.dist + exit.time(),
                path: path,
            }));
        }
    }

    // TODO: warps
}

pub fn access<W, T, E>(world: &W, ctx: &ContextWrapper<T>) -> HashMap<E::SpotId, i32>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit + Accessible<Context = T>,
{
    let mut spot_heap = BinaryHeap::new();
    let mut dist_map = HashMap::new();

    dist_map.insert(ctx.get().position(), 0);

    expand(
        world,
        ctx.get(),
        SpotAccess {
            id: ctx.get().position(),
            dist: 0,
            path: vec![],
        },
        &dist_map,
        &mut spot_heap,
    );
    println!("{:#?}", spot_heap);
    while !spot_heap.is_empty() {
        let spot_found = spot_heap.pop().unwrap().0;
        if !dist_map.contains_key(&spot_found.id) {
            dist_map.insert(spot_found.id, spot_found.dist);
            // TODO: do we need to update ctx.position for can_access rules?
            let mut c = ctx.get().clone();
            c.set_position(spot_found.id);
            expand(world, &c, spot_found, &dist_map, &mut spot_heap);
        }
    }

    dist_map
}
