use crate::context::Ctx;
use crate::world::*;
use crate::{new_hashmap, new_hashset, CommonHasher};
use pheap::PairingHeap;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct CondensedEdge<T: Ctx, S, E> {
    pub dst: S,
    pub time: u32,
    pub movement: Option<T::MovementState>,
    pub reqs: Vec<E>,
}

impl<T, S, E> CondensedEdge<T, S, E>
where
    T: Ctx,
    S: Id,
    E: Id,
{
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
        self.reqs.iter().all(|&e| world.get_exit(e).can_access(ctx))
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
enum HeapEdge<M, S, E> {
    Base(S, u32),
    Move(M, S, u32),
    Exit(E),
}

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
    let mut condensed = new_hashmap();

    for &start in world.get_all_spots() {
        // we should first do base movement so we are sure we only have one per pair
        let mut heap = PairingHeap::new();
        heap.insert((start, Vec::new()), 0);
        let mut best = new_hashmap();

        while let Some(((cur, path), t)) = heap.delete_min() {
            if !best.contains_key(&cur) {
                best.insert(cur, (path.clone(), t));
                for &local_dst in world.get_area_spots(cur) {
                    let dist = world.base_distance(cur, local_dst);
                    if dist < u32::MAX {
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
                    movement: None,
                    reqs: Vec::new(),
                };
                condensed.insert(start, vec![ce]);
            }
        }

        // each el in the heap is a path with priority equal to the time cost
        // for convenience, also add the current spot
        let mut heap = PairingHeap::new();
        heap.insert((start, Vec::<HeapEdge<T::MovementState, S, E>>::new()), 0);
        // each edge can only be used once.
        let mut base_edges_seen = new_hashset();
        let mut mvmts_edges_seen = new_hashset();
        let mut exits_seen = new_hashset();

        while let Some(((cur, path), t)) = heap.delete_min() {
            // 1. is this nonempty path to an interesting node?
            //    if so, generate its requirements and store it if no other stored
            //    edge to that node has a superset of the requirements and less time
            if world.spot_of_interest(cur) && !path.is_empty() {
                let mut moves = path.iter().filter_map(|he| {
                    if let HeapEdge::Move(m, ..) = he {
                        Some(*m)
                    } else {
                        None
                    }
                });
                let movement = if let Some(mut m) = moves.next() {
                    for m2 in moves {
                        m = T::combine(m, m2);
                    }
                    Some(m)
                } else {
                    None
                };
                let has_exit = path.iter().any(|he| matches!(he, HeapEdge::Exit(_)));
                let exits: Vec<_> = path
                    .into_iter()
                    .filter_map(|he| {
                        if let HeapEdge::Exit(e) = he {
                            if W::Exit::always(e) {
                                Some(e)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                let ce = CondensedEdge {
                    dst: cur,
                    time: t,
                    movement,
                    reqs: exits,
                };
                if let Some(vec) = condensed.get_mut(&start) {
                    if movement.is_some() || !ce.reqs.is_empty() {
                        if vec.iter().all(|c| c.dst != cur || !ce.is_subset_of(c)) {
                            vec.push(ce);
                        }
                    } else if has_exit {
                        // We have found another edge with no requirements, we can potentially shorten it
                        if let Some(first) = vec
                            .iter_mut()
                            .find(|c| c.dst == cur && c.movement.is_none() && c.reqs.is_empty())
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
            //
            // 2. Insert movements to area spots.
            for &local_dst in world.get_area_spots(cur) {
                let best = W::best_movements(start, local_dst);
                if let Some(free) = best.0 {
                    if !base_edges_seen.contains(&(cur, local_dst)) {
                        let mut p2 = path.clone();
                        p2.push(HeapEdge::Base(local_dst, free));
                        heap.insert((local_dst, p2), t + free);
                        base_edges_seen.insert((cur, local_dst));
                    }
                }
                for (m, mt) in best.1 {
                    if !mvmts_edges_seen.contains(&(cur, local_dst)) {
                        let mut p2 = path.clone();
                        p2.push(HeapEdge::Move(m, local_dst, mt));
                        heap.insert((local_dst, p2), t + mt);
                        mvmts_edges_seen.insert((cur, local_dst));
                    }
                }
            }
            // 3. Insert exits to area spots.
            for e in world.get_spot_exits(cur) {
                if W::same_area(start, e.dest()) && !exits_seen.contains(&e.id()) {
                    let mut p2 = path.clone();
                    p2.push(HeapEdge::Exit(e.id()));
                    heap.insert((e.dest(), p2), t + e.time());
                    mvmts_edges_seen.insert((cur, e.dest()));
                    exits_seen.insert(e.id());
                }
            }
        }
    }

    println!(
        "Condensed into {} total sources, {} total edges, {} interesting spots (from {}), {} interesting edges",
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
            .sum::<usize>()
    );

    condensed
}
