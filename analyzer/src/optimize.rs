#![allow(unused)]

use crate::algo::single_step;
use crate::estimates::ContextScorer;
use crate::greedy::*;
use crate::steiner::{EdgeId, NodeId, SteinerAlgo};
use crate::world::*;
use crate::{context::*, new_hashmap};
use rayon::prelude::*;
use sort_by_derive::SortBy;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::Instant;

#[derive(SortBy)]
struct AStarHeapElement<T: Ctx> {
    #[sort_by]
    estimate: u64,
    index: usize,
    el: ContextWrapper<T>,
}

// TODO: This might be a lot of repeated work. Can we cache the fastest ctxs
// to 1..n?
fn a_star<'w, W, T, L, E, A>(
    scorer: &ContextScorer<'w, W, E::SpotId, L::LocId, EdgeId<W>, A>,
    world: &W,
    mut startctx: ContextWrapper<T>,
    required: &[L::LocId],
    mut max_time: u32,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
    A: SteinerAlgo<NodeId<W>, EdgeId<W>>,
{
    let mut heap = BinaryHeap::new();
    let mut v: Vec<_> = required.iter().collect();
    v.sort_unstable();
    let get_estimate = |ctx: &ContextWrapper<T>| -> u64 {
        let remaining: Vec<_> = v
            .iter()
            .filter_map(|&&loc_id| {
                if ctx.get().todo(loc_id) {
                    Some(loc_id)
                } else {
                    None
                }
            })
            .collect();
        if remaining.is_empty() {
            ctx.elapsed().into()
        } else {
            <u32 as Into<u64>>::into(ctx.elapsed())
                + scorer.estimate_time_to_get(ctx.get(), remaining)
        }
    };
    startctx.get_mut().reset(required[0]);

    // TODO: use the seendb instead of a separate hashmap
    let mut state_map = new_hashmap();
    state_map.insert(startctx.get().clone(), startctx.elapsed());
    let greedy = if required.len() == 1 {
        match first_spot_with_locations_after_actions(world, startctx.clone(), 4, max_time) {
            Ok(mut c) => {
                grab_all(world, &mut c);
                let est = get_estimate(&startctx);
                max_time = c.elapsed();
                Some(c)
            }
            Err(c) => panic!(
                "Never found a path to {:?}!\n{}",
                required[0],
                c.history_summary()
            ),
        }
    } else {
        None
    };
    let start = Instant::now();
    // TODO: don't use an a* search for this, use the greedy search
    // take the best of {no actions, 1 action, 2 actions, etc}
    heap.push(Reverse(AStarHeapElement {
        estimate: get_estimate(&startctx),
        el: startctx,
        index: 0,
    }));
    while let Some(Reverse(AStarHeapElement {
        estimate,
        index,
        el,
    })) = heap.pop()
    {
        if required.iter().all(|&loc_id| el.get().visited(loc_id)) {
            return Some(el);
        }
        heap.extend(
            single_step(world, el, max_time)
                .into_iter()
                .filter_map(|mut ctx| {
                    match ctx.history_rev().next() {
                        Some(History::Get(_, loc_id)) => {
                            // Immediately after we visit a required location, unskip the next one
                            if index + 1 < required.len() {
                                ctx.get_mut().reset(required[index + 1]);
                            }
                            let estimate = get_estimate(&ctx);
                            if estimate > max_time.into() {
                                return None;
                            }
                            if let Some(time) = state_map.get(ctx.get()) {
                                if ctx.elapsed() > *time {
                                    return None;
                                }
                            }
                            state_map.insert(ctx.get().clone(), ctx.elapsed());
                            Some(Reverse(AStarHeapElement {
                                estimate,
                                index: index + 1,
                                el: ctx,
                            }))
                        }
                        None => None,
                        Some(h) => {
                            let estimate = get_estimate(&ctx);
                            if estimate > max_time.into() {
                                return None;
                            }
                            if let Some(time) = state_map.get(ctx.get()) {
                                if ctx.elapsed() > *time {
                                    return None;
                                }
                            }
                            state_map.insert(ctx.get().clone(), ctx.elapsed());
                            Some(Reverse(AStarHeapElement {
                                estimate,
                                index,
                                el: ctx,
                            }))
                        }
                    }
                }),
        );
    }
    match greedy {
        Some(mut c) => {
            let est = get_estimate(&c);
            println!(
                "But greedy found this path in {}:\n{}",
                c.elapsed(),
                c.history_str()
            );
            if est > c.elapsed().into() {
                println!("Overestimated!\n{}", c.history_str());
            }
            max_time = c.elapsed();
        }
        _ => (),
    }

    None
}

pub fn optimize<'w, W, T, L, E, A>(
    scorer: &ContextScorer<'w, W, E::SpotId, L::LocId, EdgeId<W>, A>,
    world: &W,
    startctx: &T,
    unique_history: Vec<HistoryAlias<T>>,
) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
    A: SteinerAlgo<NodeId<W>, EdgeId<W>> + Sync,
{
    let mut locs_required: Vec<L::LocId> = unique_history
        .into_iter()
        .filter_map(|h| match h {
            History::Get(_, loc_id) => Some(loc_id),
            History::MoveGet(_, exit_id) => Some(world.get_exit(exit_id).loc_id().unwrap()),
            _ => None,
        })
        .collect();
    println!("Optimizing route: {:?}", locs_required);

    let mut ctx = ContextWrapper::new(startctx.clone());
    for loc in world.get_all_locations() {
        ctx.get_mut().skip(loc.id());
    }
    let mut best = Vec::with_capacity(locs_required.len() + 1);
    best.push(ctx);
    // loc_history is essentially a list of locations in order to grab.
    // We go through them in order always here.
    // Grow the list of best states by finding the best route to the next point
    // from a max depth of previous states (currently 3).
    // i.e.:
    // 0 -> 1 -> 2 (essentially a greedy search with specific targets in mind)
    // 0 ------> 2 A* using the Steiner tree estimates for 1+2
    // With the best for 2, we can calc: 2 -> 3, 1 -> 3, 0 -> 3 similarly
    for next in 0..locs_required.len() {
        println!(
            "Optimizing route to loc {} of {}: {:?}",
            next + 1,
            locs_required.len(),
            locs_required[next]
        );
        let start = Instant::now();
        let g = a_star(
            scorer,
            world,
            best[next].clone(),
            &locs_required[next..=next],
            u32::MAX,
        )
        .expect("Couldn't get to next destination");
        // TODO: should max_time be an atomic? Threads would be able to update each other.
        // We can be clever and hold off on updating the actual best entry
        let mut max_time = g.elapsed();
        best.push(g);
        // 0: we are measuring 0 -> 1, 0..0 means no iters
        // 1: 1->2, so down here we want 0 -> 2, i.e. 0..1
        // 2: 2->3, so we want 1 -> 3 and 0 -> 3 (if we do depth=3). i.e. 0..2
        // 3: 3->4, we only want 1 -> 4, so prev is 1..3
        // in other words, min(next - 2, 0)..next
        let min_index = std::cmp::max(2, next) - 2;
        if let Some(ctx) = (min_index..next)
            .into_par_iter()
            .filter_map(|prev| {
                a_star(
                    scorer,
                    world,
                    best[prev].clone(),
                    &locs_required[prev..=next],
                    max_time,
                )
            })
            .min_by_key(|c| c.elapsed())
        {
            if ctx.elapsed() < max_time {
                max_time = ctx.elapsed();
                best[next + 1] = ctx;
            }
        }
        println!("This optimize round took {:?}", start.elapsed());
    }
    best
}
