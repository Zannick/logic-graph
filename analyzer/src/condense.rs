use crate::context::Ctx;
use crate::world::*;
use crate::{new_hashmap, CommonHasher};
use log;
use pheap::PairingHeap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Instant;

#[derive(Clone, Debug)]
struct Requirements<T: Ctx, E> {
    movement: Option<T::MovementState>,
    reqs: Vec<E>,
}

impl<T, E> Default for Requirements<T, E>
where
    T: Ctx,
{
    fn default() -> Self {
        Self {
            movement: None,
            reqs: Vec::new(),
        }
    }
}

impl<T, E> Requirements<T, E>
where
    T: Ctx,
    E: Id,
{
    fn add_movement(&mut self, mvmt: T::MovementState) {
        self.movement = Some(if let Some(m) = self.movement {
            T::combine(m, mvmt)
        } else {
            mvmt
        });
    }

    fn add_exit(&mut self, exit: E) {
        if !self.reqs.contains(&exit) {
            self.reqs.push(exit);
        }
    }

    fn is_subset_of(&self, ce: &Self) -> bool {
        // reqs is a subset of ce if all of reqs are in ce.reqs
        if self.reqs.iter().all(|e| ce.reqs.contains(e)) {
            // base movement is a subset of everything
            if let Some(m) = self.movement {
                // nothing (else) is a subset of base movement
                if let Some(m2) = ce.movement {
                    T::is_subset(m, m2)
                } else {
                    false
                }
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn can_access<W>(&self, world: &W, ctx: &T, movements: T::MovementState) -> bool
    where
        W: World,
        T: Ctx<World = W>,
        W::Exit: Exit<ExitId = E>,
        W::Location: Location<Context = T>,
    {
        if let Some(m) = &self.movement {
            if !T::is_subset(*m, movements) {
                return false;
            }
        }
        self.reqs
            .iter()
            .all(|&e| world.get_exit(e).can_access(ctx, world))
    }

    pub fn observe_access<W>(&self, world: &W, ctx: &T, movements: T::MovementState, observer: &mut T::Observer) -> bool
    where
        W: World,
        T: Ctx<World = W>,
        W::Exit: Exit<ExitId = E>,
        W::Location: Location<Context = T>,
    {
        if let Some(m) = &self.movement {
            if !T::is_subset(*m, movements) {
                return false;
            }
        }
        self.reqs
            .iter()
            .all(|&e| world.get_exit(e).observe_access(ctx, world, observer))
    }

    pub fn explain<W>(&self, world: &W, ctx: &T, movements: T::MovementState) -> String
    where
        W: World,
        T: Ctx<World = W>,
        W::Exit: Exit<ExitId = E>,
        W::Location: Location<Context = T>,
    {
        if let Some(m) = &self.movement {
            if !T::is_subset(*m, movements) {
                return format!(
                    "Missing required movements on this route: {:?} but need {:?}",
                    movements, m
                );
            }
        }
        for req in &self.reqs {
            let e = world.get_exit(*req);
            if !e.can_access(ctx, world) {
                return e.explain(ctx, world);
            }
        }
        String::from("CE Success")
    }

    pub fn is_empty(&self) -> bool {
        self.movement.is_none() && self.reqs.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct CondensedEdge<T: Ctx, S, E> {
    pub dst: S,
    pub time: u32,
    reqs: Requirements<T, E>,
}

impl<T, S, E> CondensedEdge<T, S, E>
where
    T: Ctx,
    E: Id,
{
    pub fn can_access<W>(&self, world: &W, ctx: &T, movements: T::MovementState) -> bool
    where
        W: World,
        T: Ctx<World = W>,
        W::Exit: Exit<ExitId = E>,
        W::Location: Location<Context = T>,
    {
        self.reqs.can_access(world, ctx, movements)
    }

    pub fn observe_access<W>(&self, world: &W, ctx: &T, movements: T::MovementState, observer: &mut T::Observer) -> bool
    where
        W: World,
        T: Ctx<World = W>,
        W::Exit: Exit<ExitId = E>,
        W::Location: Location<Context = T>,
    {
        self.reqs.observe_access(world, ctx, movements, observer)
    }

    pub fn explain<W>(&self, world: &W, ctx: &T, movements: T::MovementState) -> String
    where
        W: World,
        T: Ctx<World = W>,
        W::Exit: Exit<ExitId = E>,
        W::Location: Location<Context = T>,
    {
        self.reqs.explain(world, ctx, movements)
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum HeapEdge<M, S, E> {
    Base(S, u32),
    Move(M, S, u32),
    Exit(E),
}

/// Creates a map of condensed edges, keyed by SpotIds.
///
/// Not every spot may be represented in the map, but every value is guaranteed to be non-empty.
pub fn condense_graph<T, W, S, E>(
    world: &W,
) -> HashMap<S, Vec<CondensedEdge<T, S, E>>, CommonHasher>
where
    T: Ctx<World = W>,
    W: World,
    W::Location: Location<Context = T>,
    W::Exit: Exit<Context = T, ExitId = E, SpotId = S>,
    S: Id,
    E: Id,
{
    let start = Instant::now();
    let mut condensed: HashMap<S, Vec<CondensedEdge<T, S, E>>, CommonHasher> = new_hashmap();

    for &start in world.get_all_spots() {
        // we should first do base movement so we are sure we only have one per pair
        let mut heap = PairingHeap::new();
        heap.insert((start, Vec::new()), 0);
        let mut best = new_hashmap();

        while let Some(((cur, path), t)) = heap.delete_min() {
            if let std::collections::hash_map::Entry::Vacant(e) = best.entry(cur) {
                e.insert((path.clone(), t));
                if world.spot_of_interest(cur) && !path.is_empty() {
                    // don't travel through spots of interest
                    continue;
                }
                for &local_dst in world.get_area_spots(cur) {
                    if let Some(dist) = W::free_movement(cur, local_dst) {
                        let mut p2 = path.clone();
                        p2.push(local_dst);
                        heap.insert((local_dst, p2), t + dist);
                    }
                }
            }
        }

        for (dst, (path, time)) in best {
            if !path.is_empty() && world.spot_of_interest(dst) {
                let ce = CondensedEdge {
                    dst,
                    time,
                    reqs: Requirements::default(),
                };
                if let Some(v) = condensed.get_mut(&start) {
                    v.push(ce);
                } else {
                    condensed.insert(start, vec![ce]);
                }
            }
        }

        // each el in the heap is a path with priority equal to the time cost
        // for convenience, also add the current spot
        let mut heap = PairingHeap::new();
        heap.insert(
            (
                start,
                Vec::<HeapEdge<T::MovementState, S, E>>::new(),
                Requirements::<T, E>::default(),
            ),
            0,
        );
        // We keep the DAG from infinite iteration by checking the requirements
        // used in each path at each step; if any previous path reached that point
        // with a subset of the requirements, we don't move it forward.
        let mut known_paths: HashMap<S, Vec<Requirements<T, E>>, CommonHasher> = new_hashmap();

        while let Some(((cur, path, reqs), t)) = heap.delete_min() {
            // 0. Have we already seen a path to this point with compatible reqs?
            if let Some(r2_list) = known_paths.get_mut(&cur) {
                if r2_list.iter().any(|r2| r2.is_subset_of(&reqs)) {
                    // Implicit guarantee from the heap that we have only seen better paths,
                    // so we don't actually need to check the time.
                    continue;
                } else {
                    r2_list.push(reqs.clone());
                }
            } else {
                known_paths.insert(cur, vec![reqs.clone()]);
            }

            // 1. is this nonempty path to an interesting node?
            //    if so, generate its requirements and store it if no other stored
            //    edge to that node has a subset of the requirements and less time
            if world.spot_of_interest(cur) && !path.is_empty() {
                let has_exit = path.iter().any(|he| matches!(he, HeapEdge::Exit(_)));

                let ce = CondensedEdge {
                    dst: cur,
                    time: t,
                    reqs,
                };
                if let Some(vec) = condensed.get_mut(&start) {
                    if !ce.reqs.is_empty() {
                        // If none of the existing edges for this connection are both:
                        // a. a subset of this connection's reqs
                        // b. a better time
                        // then we can save the edge
                        if !vec
                            .iter()
                            .filter(|c| c.dst == cur)
                            .any(|c| c.reqs.is_subset_of(&ce.reqs) && ce.time < t)
                        {
                            vec.push(ce);
                        }
                    } else if has_exit {
                        // We have found another edge with no requirements, we can potentially shorten it
                        if let Some(first) =
                            vec.iter_mut().find(|c| c.dst == cur && c.reqs.is_empty())
                        {
                            if t < first.time {
                                first.time = t;
                            }
                            continue;
                        }
                        vec.push(ce);
                    }
                } else {
                    condensed.insert(start, vec![ce]);
                }

                continue;
            }
            // 2. Insert movements to area spots.
            for &local_dst in world.get_area_spots(cur) {
                if local_dst == start {
                    continue;
                }
                let best = W::best_movements(cur, local_dst);
                if let Some(free) = best.0 {
                    // Path continues with new base edge. No change to reqs
                    let mut p2 = path.clone();
                    p2.push(HeapEdge::Base(local_dst, free));
                    let r2 = reqs.clone();
                    heap.insert((local_dst, p2, r2), t + free);
                }
                for (m, mt) in best.1 {
                    // Path continues with new movement edge. Reqs update with movement
                    let mut p2 = path.clone();
                    p2.push(HeapEdge::Move(m, local_dst, mt));
                    let mut r2 = reqs.clone();
                    r2.add_movement(m);
                    heap.insert((local_dst, p2, r2), t + mt);
                }
            }
            // 3. Insert exits to area spots.
            for e in world.get_spot_exits(cur) {
                if Exit::dest(e) == start {
                    continue;
                }
                if W::same_area(start, Exit::dest(e)) {
                    // Path continues with new exit edge. Reqs update with exit.
                    let mut p2 = path.clone();
                    p2.push(HeapEdge::Exit(e.id()));
                    let mut r2 = reqs.clone();
                    r2.add_exit(e.id());
                    heap.insert((Exit::dest(e), p2, r2), t + e.base_time());
                }
            }
        }
    }

    log::info!(
        "Condensed into {} total sources, {} total edges, {} interesting spots (from {}), {} interesting edges, in {:?}",
        condensed.len(),
        condensed.values().map(|v| v.len()).sum::<usize>(),
        world
            .get_all_spots()
            .iter()
            .filter(|s| world.spot_of_interest(**s))
            .count(),
        world.get_all_spots().len(),
        condensed
            .iter()
            .filter_map(|(s, v)| if world.spot_of_interest(*s) {
                Some(v.iter().filter(|e| world.spot_of_interest(e.dst)).count())
            } else {
                None
            })
            .sum::<usize>(),
        start.elapsed(),
    );

    condensed
}
