use std::collections::HashSet;

use crate::access::*;
use crate::context::*;
use crate::minimize::*;
use crate::world::*;

pub fn first_spot_with_locations_after_actions<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_depth: i8,
) -> Result<ContextWrapper<T>, ContextWrapper<T>>
where
    W: World<Exit = E, Location = L>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = L::Currency, ExitId = L::ExitId>,
    L: Location<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = L::Currency>,
{
    let spot_map = accessible_spots(world, ctx);
    let orig_vec: Vec<ContextWrapper<T>> = spot_map.into_values().collect();
    if let Some(ctx) = orig_vec
        .iter()
        .filter(|ctx| spot_has_locations(world, ctx.get()))
        .min_by_key(|ctx| ctx.elapsed())
    {
        return Ok(ctx.clone());
    }
    let min_spot = orig_vec
        .iter()
        .min_by_key(|ctx| ctx.elapsed())
        .expect("couldn't reach any spots!")
        .clone();

    let mut useful_spots = Vec::new();
    let mut seen = HashSet::new();
    let mut to_process = orig_vec.clone();
    seen.extend(to_process.iter().map(|ctx| ctx.get().clone()));

    // Only allow global actions at the start position, or as the last action.
    // This avoids extreme fanout.
    for action in world
        .get_global_actions()
        .iter()
        .filter(|a| a.can_access(min_spot.get()))
    {
        let mut newctx = min_spot.clone();
        newctx.activate(action);
        for nextctx in accessible_spots(world, newctx).into_values() {
            if spot_has_locations(world, nextctx.get()) {
                useful_spots.push(nextctx);
            } else {
                if !seen.contains(nextctx.get()) {
                    seen.insert(nextctx.get().clone());
                    to_process.push(nextctx);
                }
            }
        }
    }

    let mut depth = 0;
    while depth < max_depth {
        let mut next_process = Vec::new();
        for spot_ctx in to_process.iter().filter(|ctx| spot_has_actions(world, ctx)) {
            for action in world
                .get_spot_actions(spot_ctx.get().position())
                .iter()
                .filter(|a| a.can_access(spot_ctx.get()))
            {
                let mut newctx = spot_ctx.clone();
                newctx.activate(action);
                for nextctx in accessible_spots(world, newctx).into_values() {
                    if spot_has_locations(world, nextctx.get()) {
                        if depth > 0 {
                            return Ok(nextctx);
                        } else {
                            useful_spots.push(nextctx);
                        }
                    } else {
                        if !seen.contains(nextctx.get()) {
                            seen.insert(nextctx.get().clone());
                            next_process.push(nextctx);
                        }
                    }
                }
            }

            // Only allow global actions at the start position, or as the last action.
            for action in world
                .get_global_actions()
                .iter()
                .filter(|a| a.can_access(spot_ctx.get()))
            {
                let mut newctx = spot_ctx.clone();
                newctx.activate(action);
                for nextctx in accessible_spots(world, newctx).into_values() {
                    if spot_has_locations(world, nextctx.get()) {
                        return Ok(nextctx);
                    }
                }
            }
        }
        if !useful_spots.is_empty() {
            break;
        }
        to_process = next_process;
        to_process.sort_unstable_by_key(|ctx| ctx.elapsed());
        depth += 1;
    }

    useful_spots
        .into_iter()
        .min_by_key(|ctx| ctx.elapsed())
        .ok_or(min_spot)
}

pub fn grab_all<W, T, L, E>(world: &W, ctx: &mut ContextWrapper<T>)
where
    W: World<Exit = E, Location = L>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let (locs, exit) = visitable_locations(world, ctx.get());
    for loc in locs {
        if ctx.get().todo(loc.id()) && loc.can_access(ctx.get()) {
            ctx.visit(world, loc);
        }
    }

    if let Some((l, e)) = exit {
        if ctx.get().todo(l) {
            let exit = world.get_exit(e);
            let loc = world.get_location(l);
            if loc.can_access(ctx.get()) && exit.can_access(ctx.get()) {
                ctx.visit_exit(world, loc, exit);
            }
        }
    }
}

fn greedy_internal<W, T, L, E>(
    world: &W,
    mut ctx: ContextWrapper<T>,
) -> Result<ContextWrapper<T>, ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    world.skip_unused_items(ctx.get_mut());
    while !world.won(ctx.get()) {
        match first_spot_with_locations_after_actions(world, ctx, 2) {
            Ok(c) => {
                ctx = c;
                grab_all(world, &mut ctx);
            }
            Err(c) => return Err(c),
        }
    }
    Ok(ctx)
}

pub fn greedy_search<W, T, L, E>(
    world: &W,
    ctx: &ContextWrapper<T>,
) -> Result<ContextWrapper<T>, ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    greedy_internal(world, ctx.clone())
}

pub fn greedy_search_from<W, T, L, E>(
    world: &W,
    ctx: &T,
) -> Result<ContextWrapper<T>, ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    greedy_internal(world, ContextWrapper::new(ctx.clone()))
}

pub fn minimize_greedy<W, T, L, E>(
    world: &W,
    startctx: &T,
    wonctx: &ContextWrapper<T>,
) -> ContextWrapper<T>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let ctx = minimize(world, startctx, wonctx);
    greedy_search(world, &ctx).expect("Couldn't beat game after minimizing!")
}

pub fn minimal_greedy_playthrough<W, T, L, E>(
    world: &W,
    ctx: &ContextWrapper<T>,
) -> ContextWrapper<T>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let wonctx = greedy_search(world, ctx).expect("Didn't win with greedy search");
    minimize_greedy(world, ctx.get(), &wonctx)
}
