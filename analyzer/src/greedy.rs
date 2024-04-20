use crate::access::*;
use crate::context::*;
use crate::new_hashset;
use crate::world::*;
use std::collections::HashSet;

pub fn first_spot_with_locations_after_actions<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_depth: i8,
    max_time: u32,
) -> Result<ContextWrapper<T>, ContextWrapper<T>>
where
    W: World<Exit = E, Location = L>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = L::Currency, ExitId = L::ExitId>,
    L: Location<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = L::Currency>,
{
    let spot_map = accessible_spots(world, ctx, max_time, false);
    let mut orig_vec: Vec<ContextWrapper<T>> = spot_map.into_values().collect();
    orig_vec.sort_unstable_by_key(|ctx| ctx.elapsed());
    if let Some(ctx) = orig_vec
        .iter()
        .find(|ctx| spot_has_locations(world, ctx.get()))
    {
        return Ok(ctx.clone());
    }
    let min_spot = orig_vec.first().expect("couldn't reach any spots!").clone();
    let max_spot = orig_vec.last().expect("couldn't reach any spots!");
    // Don't allow going all the way there then all the way back again.
    let max_time = 2 * max_spot.elapsed() - min_spot.elapsed();

    let mut useful_spots = Vec::new();
    let mut seen = new_hashset();
    let mut to_process: Vec<_> = orig_vec
        .iter()
        .map(|c| (c.clone(), HashSet::new()))
        .collect();
    seen.extend(to_process.iter().map(|(ctx, _)| ctx.get().clone()));

    // Only allow global actions once each.
    // This avoids extreme fanout.
    let mut depth = 0;
    while depth < max_depth && !to_process.is_empty() {
        let mut next_process = Vec::new();
        for (spot_ctx, used_globals) in to_process
            .iter()
            .filter(|(ctx, _)| spot_has_actions(world, ctx.get()))
        {
            for action in world
                .get_spot_actions(spot_ctx.get().position())
                .iter()
                .filter(|a| !used_globals.contains(&a.id()) && a.can_access(spot_ctx.get(), world))
            {
                let mut newctx = spot_ctx.clone();
                newctx.activate(world, action);
                for nextctx in accessible_spots(world, newctx, max_time, false).into_values() {
                    if spot_has_locations(world, nextctx.get()) {
                        if depth > 0 {
                            return Ok(nextctx);
                        } else {
                            useful_spots.push(nextctx);
                        }
                    } else if !seen.contains(nextctx.get()) {
                        seen.insert(nextctx.get().clone());
                        next_process.push((nextctx, used_globals.clone()));
                    }
                }
            }

            // Only allow global actions once each.
            for action in world
                .get_global_actions()
                .iter()
                .filter(|a| a.can_access(spot_ctx.get(), world))
            {
                let mut newctx = spot_ctx.clone();
                newctx.activate(world, action);
                for nextctx in accessible_spots(world, newctx, max_time, false).into_values() {
                    if spot_has_locations(world, nextctx.get()) {
                        return Ok(nextctx);
                    } else if !seen.contains(nextctx.get()) {
                        let mut next_globals = used_globals.clone();
                        next_globals.insert(action.id());
                        seen.insert(nextctx.get().clone());
                        next_process.push((nextctx, next_globals));
                    }
                }
            }
        }
        if !useful_spots.is_empty() {
            break;
        }
        to_process = next_process;
        to_process.sort_unstable_by_key(|(ctx, _)| ctx.elapsed());
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
    let mut hybrids = Vec::new();
    for loc in world.get_spot_locations(ctx.get().position()) {
        if let Some(e) = loc.exit_id() {
            hybrids.push((loc, world.get_exit(*e)));
        } else {
            if ctx.get().todo(loc) && loc.can_access(ctx.get(), world) {
                ctx.visit(world, loc);
            }
        }
    }

    if let Some((loc, exit)) = hybrids
        .into_iter()
        .filter(|(loc, exit)| {
            ctx.get().todo(loc)
                && loc.can_access(ctx.get(), world)
                && exit.can_access(ctx.get(), world)
        })
        .min_by_key(|(loc, exit)| loc.time(ctx.get(), world) + exit.time(ctx.get(), world))
    {
        ctx.visit_exit(world, loc, exit);
    }
}

fn greedy_internal<W, T, L, E>(
    world: &W,
    mut ctx: ContextWrapper<T>,
    max_time: u32,
    max_depth: i8,
) -> Result<ContextWrapper<T>, ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    while !world.won(ctx.get()) {
        if ctx.elapsed() > max_time {
            return Err(ctx);
        }
        match first_spot_with_locations_after_actions(world, ctx, max_depth, max_time) {
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
    max_time: u32,
    max_depth: i8,
) -> Result<ContextWrapper<T>, ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    greedy_internal(world, ctx.clone(), max_time, max_depth)
}

pub fn greedy_search_from<W, T, L, E>(
    world: &W,
    ctx: &T,
    max_time: u32,
) -> Result<ContextWrapper<T>, ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    greedy_internal(world, ContextWrapper::new(ctx.clone()), max_time, 2)
}
