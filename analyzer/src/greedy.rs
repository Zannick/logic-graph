use indextree::NodeId;
use std::collections::HashMap;

use crate::access::*;
use crate::context::*;
use crate::history::HistoryTree;
use crate::minimize::*;
use crate::world::*;

pub fn nearest_spot_with_checks<W, T, E, L, Wp>(
    world: &W,
    tree: &HistoryTree<T>,
    spot_map: &HashMap<E::SpotId, NodeId>,
) -> Option<NodeId>
where
    W: World<Exit = E, Location = L, Warp = Wp>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit<Context = T> + Accessible<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId>,
{
    if let Some((_, &node)) = spot_map
        .iter()
        .filter(|(_, &node)| spot_has_locations(world, tree.get(node).get()))
        .min_by_key(|(s, &node)| (tree.get(node).elapsed(), **s))
    {
        Some(node)
    } else {
        None
    }
}

pub fn nearest_spot_with_actions<W, T, E, L, Wp>(
    world: &W,
    tree: &HistoryTree<T>,
    spot_map: &HashMap<E::SpotId, NodeId>,
) -> Option<NodeId>
where
    W: World<Exit = E, Location = L, Warp = Wp>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit<Context = T> + Accessible<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId>,
{
    if let Some((_, &node)) = spot_map
        .iter()
        .filter(|(_, &node)| spot_has_actions(world, tree, node, tree.get(node)))
        .min_by_key(|(s, &node)| (tree.get(node).elapsed(), **s))
    {
        Some(node.clone())
    } else {
        None
    }
}

pub fn grab_all<W, T, L, E>(world: &W, tree: &HistoryTree<T>, current: NodeId) -> NodeId
where
    W: World<Exit = E, Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    E: Exit<Context = T, ExitId = L::ExitId, Currency = L::Currency>,
{
    let ctx = tree.get(current);
    let (locs, exit) = visitable_locations(world, ctx.get());
    for loc in locs {
        if ctx.get().todo(loc.id()) {
            let mut newctx = ctx.clone();
            let step = newctx.visit(world, loc);
            current = tree.insert_and_get(current, step, newctx);
            ctx = tree.get(current);
        }
    }

    if let Some((l, e)) = exit {
        if ctx.get().todo(l) {
            let mut newctx = ctx.clone();
            let exit = world.get_exit(e);
            let loc = world.get_location(l);
            let step = newctx.visit_exit(world, loc, exit);
            current = tree.insert_and_get(current, step, newctx);
            ctx = tree.get(current);
        }
    }
    current
}

pub fn do_all<W, T, L, E>(world: &W, tree: &HistoryTree<T>, current: NodeId) -> NodeId
where
    W: World<Exit = E, Location = L>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit<Context = T> + Accessible<Context = T, Currency = L::Currency>,
{
    let ctx = tree.get(current);
    for act in world
        .get_global_actions()
        .iter()
        .chain(world.get_spot_actions(ctx.get().position()))
    {
        if act.can_access(ctx.get()) && ctx.is_useful(tree, current, act) {
            let newctx = ctx.clone();
            let step = newctx.activate(act);
            current = tree.insert_and_get(current, step, newctx);
            ctx = tree.get(current);
        }
    }
    current
}

pub fn greedy_search<W, T, L, E>(
    world: &W,
    tree: &mut HistoryTree<T>,
    start: NodeId,
) -> Option<NodeId>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T, Currency = L::Currency>,
{
    let node = start;
    let ctx = tree.get(node);
    while !world.won(ctx.get()) {
        let spot_map = accessible_spots(world, tree, node);
        if let Some(n) = nearest_spot_with_checks(world, tree, &spot_map) {
            node = n;
            grab_all(world, tree, node);
        } else if let Some(n) = nearest_spot_with_actions(world, tree, &spot_map) {
            node = n;
            // TODO: this probably shouldn't do all global actions, maybe we pick the fastest/cheapest?
            do_all(world, tree, node);
        } else {
            return None;
        }
        ctx = tree.get(node);
    }
    Some(node)
}

pub fn minimize_greedy<W, T, L, E>(
    world: &W,
    tree: &mut HistoryTree<T>,
    startctx: &T,
    winner: NodeId,
) -> NodeId
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T, Currency = L::Currency>,
{
    let ctx = minimize(world, startctx, tree.get(winner));
    // Since the tree has the skip status embedded, we have multiple disjoint trees.
    // However, we can copy at least the winning history into a new tree using the new
    // starting context
    let (start, _) = tree.insert_tree(world, &tree.get_history(winner), &ctx);
    greedy_search(world, tree, start).expect("Couldn't beat game after minimizing!")
}

pub fn minimal_greedy_playthrough<W, T, L, E>(
    world: &W,
    tree: &mut HistoryTree<T>,
    ctx: &ContextWrapper<T>,
) -> NodeId
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T, Currency = L::Currency>,
{
    let startctx = ctx.clone();
    let start = tree.new_tree(ctx.last, ctx.clone());
    let winner = greedy_search(world, tree, start).expect("Didn't win with greedy search");
    minimize_greedy(world, tree, startctx.get(), winner)
}
