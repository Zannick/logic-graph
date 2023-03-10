use std::collections::HashMap;

use crate::access::*;
use crate::context::*;
use crate::minimize::*;
use crate::world::*;

pub fn nearest_spot_with_checks<W, T, E, L, Wp>(
    world: &W,
    spot_map: &HashMap<E::SpotId, ContextWrapper<T>>,
) -> Option<ContextWrapper<T>>
where
    W: World<Exit = E, Location = L, Warp = Wp>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId>,
{
    if let Some((_, ctx)) = spot_map
        .iter()
        .filter(|(_, ctx)| spot_has_locations(world, ctx.get()))
        .min_by_key(|(s, c)| (c.elapsed(), **s))
    {
        Some(ctx.clone())
    } else {
        None
    }
}

pub fn nearest_spot_with_actions<W, T, E, L, Wp>(
    world: &W,
    spot_map: &HashMap<E::SpotId, ContextWrapper<T>>,
) -> Option<ContextWrapper<T>>
where
    W: World<Exit = E, Location = L, Warp = Wp>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId>,
{
    if let Some((_, ctx)) = spot_map
        .iter()
        .filter(|(_, ctx)| spot_has_actions(world, ctx))
        .min_by_key(|(s, c)| (c.elapsed(), **s))
    {
        Some(ctx.clone())
    } else {
        None
    }
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
        if ctx.get().todo(loc.id()) {
            ctx.visit(world, loc);
        }
    }

    if let Some((l, e)) = exit {
        if ctx.get().todo(l) {
            let exit = world.get_exit(e);
            let loc = world.get_location(l);
            ctx.visit_exit(world, loc, exit);
        }
    }
}

pub fn do_all<W, T, L, E>(world: &W, ctx: &mut ContextWrapper<T>)
where
    W: World<Exit = E, Location = L>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    for act in world
        .get_global_actions()
        .iter()
        .chain(world.get_spot_actions(ctx.get().position()))
    {
        if act.can_access(ctx.get()) && ctx.is_useful(act) {
            ctx.activate(act);
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
    let mut actions = 0;
    while !world.won(ctx.get()) {
        let spot_map = accessible_spots(world, ctx);
        if let Some(c) = nearest_spot_with_checks(world, &spot_map) {
            ctx = c;
            grab_all(world, &mut ctx);
            actions = 0;
        } else if let Some(c) = nearest_spot_with_actions(world, &spot_map) {
            ctx = c;
            // TODO: this probably shouldn't do all global actions, maybe we pick the fastest/cheapest?
            do_all(world, &mut ctx);
            actions += 1;
        } else {
            let ctx = spot_map
                .into_values()
                .into_iter()
                .last()
                .expect("couldn't reach any spots!");
            return Err(ctx);
        }
        if actions > 2 {
            let ctx = spot_map
                .into_values()
                .into_iter()
                .last()
                .expect("couldn't reach any spots!");
            return Err(ctx);
        }
    }
    Ok(ctx)
}

pub fn greedy_search<W, T, L, E>(world: &W, ctx: &ContextWrapper<T>) -> Result<ContextWrapper<T>, ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    greedy_internal(world, ctx.clone())
}

pub fn greedy_search_from<W, T, L, E>(world: &W, ctx: &T) -> Result<ContextWrapper<T>, ContextWrapper<T>>
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
