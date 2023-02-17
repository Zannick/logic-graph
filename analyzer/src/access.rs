use crate::context::*;
use crate::world::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fmt::Debug;

#[derive(Clone, Eq, Debug)]
pub struct SpotAccess<S, T>
where
    S: Id,
    T: Ctx<SpotId = S>,
{
    pub dist: i32,
    pub id: S,
    // this could store info about the path to the spot for better history
    pub path: Vec<History<T>>,
}
impl<S, T> Ord for SpotAccess<S, T>
where
    S: Id,
    T: Ctx<SpotId = S>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        // reversed for min-heap
        other.dist.cmp(&self.dist)
    }
}
impl<S, T> PartialOrd for SpotAccess<S, T>
where
    S: Id,
    T: Ctx<SpotId = S>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<S, T> PartialEq for SpotAccess<S, T>
where
    S: Id,
    T: Ctx<SpotId = S>,
{
    fn eq(&self, other: &Self) -> bool {
        self.dist == other.dist
    }
}

pub fn expand<T, S, E>(
    world: &impl World<Context = T, SpotId = S, Exit = impl Exit<ExitId = E, SpotId = S> + Accessible<Context = T>>,
    ctx: &T,
    start: SpotAccess<S, T>,
    dist_map: &HashMap<S, i32>,
    spot_heap: &mut BinaryHeap<SpotAccess<S, T>>,
) where
    T: Ctx<SpotId = S, ExitId = E>,
    S: Id,
    E: Id,
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
            spot_heap.push(SpotAccess {
                id: *spot,
                // if we use ctx.position for allowed movements we need to set it
                dist: start.dist + ctx.local_travel_time(start.id, *spot),
                path: path,
            });
        }
    }

    for exit in world.get_spot_exits(start.id) {
        if !dist_map.contains_key(&exit.dest()) && exit.can_access(ctx) {
            let mut path = start.path.clone();
            path.push(History::Move(exit.id()));
            spot_heap.push(SpotAccess {
                id: exit.dest(),
                dist: start.dist + exit.time(),
                path: path,
            });
        }
    }

    // TODO: warps
}

pub fn access<T, S, E>(
    world: &impl World<Context = T, SpotId = S, Exit = impl Exit<ExitId = E, SpotId = S> + Accessible<Context = T>>,
    ctx: &ContextWrapper<T>,
) -> HashMap<S, i32>
where
    T: Ctx<SpotId = S, ExitId = E> + Debug,
    S: Id,
    E: Id,
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
        let spot_found = spot_heap.pop().expect("nonempty");
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
