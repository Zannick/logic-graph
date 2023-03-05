#![allow(unused_variables)]

use crate::access::*;
use crate::context::*;
use crate::greedy::*;
use crate::heap::LimitedHeap;
use crate::history::HistoryTree;
use crate::minimize::*;
use crate::world::*;
use indextree::NodeId;
use std::fmt::Debug;

pub fn explore<W, T, L, E>(
    world: &W,
    tree: &mut HistoryTree<T>,
    current: NodeId,
    heap: &mut LimitedHeap<T>,
) -> Vec<NodeId>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<ExitId = L::ExitId, Context = T, Currency = L::Currency>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = L::Currency>,
{
    let spot_map = accessible_spots(world, tree, current);
    let mut vec: Vec<NodeId> = spot_map.into_values().collect();

    if vec.is_empty() {
        return vec;
    }

    let ctx_vec: Vec<&mut ContextWrapper<T>> =
        vec.iter().filter_map(|&node| tree.get_mut(node)).collect();
    ctx_vec.sort_unstable_by_key(|&el| el.elapsed());
    let shortest = ctx_vec[0].elapsed();
    // Suppose the distances to these spots are (delta from the first one) 0, 2, 3, 5, 10.
    // We want penalties to increase somewhat quadratically based on count (not just distance).
    // Penalties:
    // First el: 0. Second el: 0. Third el: 2nd-1st (2).
    // Fourth el: (2nd-1st)*2 + 3rd-2nd = 3rd+2nd - 2*1st, (4+1)
    // Fifth el: (3rd-1st)*2 + 4th-3rd = 4th+3rd - 2*1st, (6+2)
    // that's 0, 0, 2, 5, 8
    // penalties for 0, 1, 2, 3, 4, 5, 6: 0, 0, 1, 3, 7, 15, 31
    for i in 2..vec.len() {
        let penalty = ctx_vec[i].elapsed() + ctx_vec[i - 1].elapsed() - 2 * shortest;
        ctx_vec[i].penalize(penalty);
    }
    vec
}

pub fn visit_locations<W, T, L, E>(
    world: &W,
    tree: &mut HistoryTree<T>,
    current: NodeId,
    heap: &mut LimitedHeap<T>,
) where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T, Currency = L::Currency>,
{
    let mut node_list = vec![(current, tree.get(current))];
    let (mut locs, exit) = visitable_locations(world, tree.get(current).get());
    if locs.is_empty() && exit == None {
        return;
    }
    locs.sort_unstable_by_key(|loc| loc.time());
    for loc in locs {
        let last_ctxs = node_list;
        node_list = Vec::new();
        node_list.reserve(last_ctxs.len() * 2);
        for (current, ctx) in last_ctxs {
            if ctx.get().todo(loc.id()) && loc.can_access(ctx.get()) {
                // TODO: Add a better way to prevent this from causing too wide a branching factor
                // or remove.
                if ctx.minimize || !loc.is_free() {
                    let mut newctx = ctx.clone();
                    newctx.get_mut().skip(loc.id());
                    // Check if this loc is required. If it is, we can't skip it.
                    if can_win(world, newctx.get()) {
                        node_list.push((current, &newctx));
                    }
                }

                // Get the item and mark the location visited.
                let mut newctx = ctx.clone();
                let step = newctx.visit(world, loc);
                if let Ok(id) = tree.insert(current, step, newctx) {
                    node_list.push((id, tree.get(id)));
                }
            } else {
                node_list.push((current, ctx));
            }
        }
    }

    if let Some((l, e)) = exit {
        let exit = world.get_exit(e);
        let loc = world.get_location(l);
        let last_ctxs = node_list;
        node_list = Vec::new();
        node_list.reserve(last_ctxs.len());
        for (current, ctx) in last_ctxs {
            // Get the item and move along the exit.

            let mut newctx = ctx.clone();
            let step = newctx.visit_exit(world, loc, exit);
            if let Ok(id) = tree.insert(current, step, newctx) {
                node_list.push((id, tree.get(id)));
            }
        }
    }
    heap.extend(node_list.iter().map(|(&n, _)| n));
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
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
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
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
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

fn search_step<W, T, L, E>(world: &W, ctx: ContextWrapper<T>, heap: &mut LimitedHeap<T>)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId, LocId = E::LocId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T, Currency = L::Currency>,
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

pub fn search<W, T, L, E>(world: &W, mut ctx: T)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId, LocId = E::LocId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T, Currency = L::Currency>,
{
    world.skip_unused_items(&mut ctx);
    let mut tree = HistoryTree::new();
    let startctx = ContextWrapper::new(ctx);
    let start = tree.new_tree(startctx.last, startctx);
    let won = greedy_search(world, &mut tree, start).expect("Did not find a solution");

    let m = minimize_greedy(world, &mut tree, startctx.get(), won);

    let wctx = tree.get(won);
    let mctx = tree.get(m);

    println!(
        "Found greedy solution of {}ms, minimized to {}ms",
        wctx.elapsed(),
        mctx.elapsed()
    );
    let mut winner = if wctx.elapsed() < mctx.elapsed() {
        won
    } else {
        m
    };

    tree.insert_tree_from(world, &tree.get_history(m), start);

    let mut heap = LimitedHeap::new();
    heap.set_lenient_max_time(wctx.elapsed());
    heap.set_lenient_max_time(mctx.elapsed());
    heap.push(startctx.clone());
    println!("Max time to consider is now: {}ms", heap.max_time());
    let mut iters = 0;
    let mut m_iters = 0;

    while let Some(ctx) = heap.pop() {
        if world.won(ctx.get()) {
            println!(
                "Found winning {}path after {} rounds, in estimated {}ms, with {} remaining in heap",
                if ctx.minimize { "*minimized* " } else { "" },
                iters,
                ctx.elapsed(),
                heap.len()
            );
            heap.set_lenient_max_time(ctx.elapsed());
            if !ctx.minimize {
                let mut newctx =
                    ContextWrapper::new(remove_all_unvisited(world, startctx.get(), &ctx));
                newctx.minimize = true;
                heap.push(newctx);
            }

            if ctx.elapsed() < winner.elapsed() {
                winner = ctx;
            }

            println!("Max time to consider is now: {}ms", heap.max_time());
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
                "Round {} (min: {}) (heap size {}, skipped {} pushes + {} pops):\n  {}",
                iters,
                m_iters,
                heap.len(),
                iskips,
                pskips,
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
    println!(
        "Final result: est. {}ms\n{}",
        winner.elapsed(),
        winner.history_str()
    );
}
