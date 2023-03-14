#![allow(unused_variables)]

use crate::access::*;
use crate::context::*;
use crate::greedy::*;
use crate::heap::LimitedHeap;
use crate::minimize::*;
use crate::solutions::SolutionCollector;
use crate::world::*;
use std::fmt::Debug;

pub fn explore<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    heap: &mut LimitedHeap<T>,
) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<ExitId = L::ExitId, Context = T, Currency = L::Currency>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = L::Currency>,
{
    let spot_map = accessible_spots(world, ctx);
    let mut vec: Vec<ContextWrapper<T>> = spot_map.into_values().collect();

    if vec.is_empty() {
        return vec;
    }

    vec.sort_unstable_by_key(|el| el.elapsed());
    let shortest = vec[0].elapsed();
    // Suppose the distances to these spots are (delta from the first one) 0, 2, 3, 5, 10.
    // We want penalties to increase somewhat quadratically based on count (not just distance).
    // Penalties:
    // First el: 0. Second el: 0. Third el: 2nd-1st (2).
    // Fourth el: (2nd-1st)*2 + 3rd-2nd = 3rd+2nd - 2*1st, (4+1)
    // Fifth el: (3rd-1st)*2 + 4th-3rd = 4th+3rd - 2*1st, (6+2)
    // that's 0, 0, 2, 5, 8
    // penalties for 0, 1, 2, 3, 4, 5, 6: 0, 0, 1, 3, 7, 15, 31
    for i in 2..vec.len() {
        let penalty = vec[i].elapsed() + vec[i - 1].elapsed() - 2 * shortest;
        vec[i].penalize(penalty);
    }
    vec
}

pub fn visit_locations<W, T, L, E>(world: &W, ctx: ContextWrapper<T>, heap: &mut LimitedHeap<T>)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<ExitId = L::ExitId, Context = T, Currency = L::Currency>,
{
    let mut ctx_list = vec![ctx];
    let (mut locs, exit) = visitable_locations(world, ctx_list[0].get());
    if locs.is_empty() && exit == None {
        return;
    }
    locs.sort_unstable_by_key(|loc| loc.time());
    for loc in locs {
        let last_ctxs = ctx_list;
        ctx_list = Vec::new();
        ctx_list.reserve(last_ctxs.len() * 2);
        for mut ctx in last_ctxs {
            if ctx.get().todo(loc.id()) && loc.can_access(ctx.get()) {
                // Major branching factor: sometimes we can try skipping a location:
                // 1. If location has a cost, we might not want it.
                // 2. Otherwise, if we're in minimize mode, any location is potentially skippable.
                //    But it's not worth skipping locations that are free in time and money;
                //    they come along for free with other locations, or we route differently.
                if (ctx.minimize && loc.time() > 0) || !loc.is_free() {
                    let mut newctx = ctx.clone();
                    newctx.get_mut().skip(loc.id());
                    // Check if this loc is required. If it is, we can't skip it.
                    if can_win(world, newctx.get()).is_ok() {
                        ctx_list.push(newctx);
                    }
                }

                // Get the item and mark the location visited.
                ctx.visit(world, loc);
            }
            ctx_list.push(ctx);
        }
    }

    if let Some((l, e)) = exit {
        let exit = world.get_exit(e);
        let loc = world.get_location(l);
        for ctx in ctx_list.iter_mut() {
            // Get the item and move along the exit.
            ctx.visit_exit(world, loc, exit);
        }
    }
    heap.extend(ctx_list);
}

pub fn action_unlocked_anything<W, T, L, E>(
    world: &W,
    ctx: &ContextWrapper<T>,
    act: &W::Action,
    spot_ctxs: &Vec<ContextWrapper<T>>,
) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
{
    // TODO: can this be cached for the next search step?
    let new_spots = accessible_spots(world, ctx.clone());
    if new_spots.len() > spot_ctxs.len() {
        return true;
    }
    let mut missing = 0;
    for spot_ctx in spot_ctxs {
        if let Some(spot_again) = new_spots.get(&spot_ctx.get().position()) {
            if spot_again.elapsed() < spot_ctx.elapsed() {
                return true;
            }
            let new_locs = all_visitable_locations(world, spot_again.get());
            let old_locs = all_visitable_locations(world, spot_ctx.get());
            if new_locs.iter().any(|loc| !old_locs.contains(&loc)) {
                return true;
            }
        } else {
            missing += 1;
            continue;
        }
    }
    // The overlap is len() - missing, so if the new count is greater, we found new spots
    new_spots.len() > spot_ctxs.len() - missing
}

pub fn activate_actions<W, T, L, E>(
    world: &W,
    ctx: &ContextWrapper<T>,
    penalty: i32,
    spot_ctxs: &Vec<ContextWrapper<T>>,
    heap: &mut LimitedHeap<T>,
) where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
{
    for act in world
        .get_global_actions()
        .iter()
        .chain(world.get_spot_actions(ctx.get().position()))
    {
        if act.can_access(ctx.get()) && ctx.is_useful(act) {
            let mut c2 = ctx.clone();
            c2.activate(act);
            if action_unlocked_anything(world, &c2, act, spot_ctxs) {
                c2.penalize(penalty);
                heap.push(c2);
            }
        }
    }
}

pub fn search_step<W, T, L, E>(world: &W, ctx: ContextWrapper<T>, heap: &mut LimitedHeap<T>)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<ExitId = L::ExitId, LocId = L::LocId, Context = T, Currency = L::Currency>,
{
    // The process will look more like this:
    // 1. explore -> vec of spot ctxs with penalties applied
    // 2. get largest dist
    // 3. (activate_actions) for each ctx, check for global actions and spot actions
    // 4. (visit_locations) for each ctx, get all available locations
    let spot_ctxs = explore(world, ctx, heap);

    if let (Some(s), Some(f)) = (spot_ctxs.first(), spot_ctxs.last()) {
        let max_dist = f.elapsed() - s.elapsed();
        for ctx in spot_ctxs.iter() {
            let has_locs = spot_has_locations(world, ctx.get());
            if spot_has_actions(world, ctx) {
                activate_actions(
                    world,
                    ctx,
                    if !has_locs { max_dist + 1000 } else { max_dist },
                    &spot_ctxs,
                    heap,
                );
            }
            if has_locs {
                visit_locations(world, ctx.clone(), heap);
            }
        }
    }
}

pub fn search<W, T, L, E>(world: &W, mut ctx: T) -> Result<(), std::io::Error>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<Context = T, ExitId = L::ExitId, LocId = L::LocId, Currency = L::Currency>,
{
    world.skip_unused_items(&mut ctx);
    let startctx = ContextWrapper::new(ctx);
    let mut heap = LimitedHeap::new();
    let mut solutions = SolutionCollector::<T>::new("data/solutions.txt").unwrap();

    match greedy_search(world, &startctx) {
        Ok(wonctx) => {
            let m = minimize_greedy(world, startctx.get(), &wonctx);
            println!(
                "Found greedy solution of {}ms, minimized to {}ms",
                wonctx.elapsed(),
                m.elapsed()
            );
            heap.set_lenient_max_time(wonctx.elapsed());
            heap.set_lenient_max_time(m.elapsed());
            solutions.insert(wonctx);
            solutions.insert(m);
        }
        Err(ctx) => {
            println!(
                "Found no greedy solution, maximal attempt reached dead-end after {}ms",
                ctx.elapsed()
            );
            heap.set_lenient_max_time(ctx.elapsed() * 2);
        }
    };
    heap.push(startctx.clone());
    println!("Max time to consider is now: {}ms", heap.max_time());
    let mut iters = 0;
    let mut m_iters = 0;

    while let Some(ctx) = heap.pop() {
        if world.won(ctx.get()) {
            let old_time = heap.max_time();
            heap.set_lenient_max_time(ctx.elapsed());

            if heap.max_time() < old_time {
                println!(
                    "Found winning {}path after {} rounds, in estimated {}ms, with {} remaining in heap",
                    if ctx.minimize { "*minimized* " } else { "" },
                    iters,
                    ctx.elapsed(),
                    heap.len()
                );
                println!("Max time to consider is now: {}ms", heap.max_time());
            }

            if !ctx.minimize {
                let mut newctx =
                    ContextWrapper::new(remove_all_unvisited(world, startctx.get(), &ctx));
                newctx.minimize = true;
                heap.push(newctx);
            }

            solutions.insert(ctx);

            continue;
        }
        if ctx.score() < -3 * heap.max_time() {
            println!(
                "Remaining items have low score: score={} vs max_time={}ms",
                ctx.score(),
                heap.max_time()
            );
            break;
        }
        iters += 1;
        if ctx.minimize {
            m_iters += 1;
        }
        if iters % 10000 == 0 {
            let (iskips, pskips) = heap.stats();
            println!(
                "Round {} (min: {}) (heap size {}, skipped {} pushes + {} pops, unique solutions: {}):\n  {}",
                iters,
                m_iters,
                heap.len(),
                iskips,
                pskips,
                solutions.unique(),
                ctx.info()
            );
        }
        search_step(world, ctx, &mut heap);
    }
    let (iskips, pskips) = heap.stats();
    println!(
        "Finished after {} rounds (w/ {} minimize rounds), skipped {} pushes + {} pops",
        iters, m_iters, iskips, pskips
    );
    solutions.export()
}
