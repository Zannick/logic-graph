use crate::access::*;
use crate::context::*;
use crate::world::*;

pub fn nearest_spot_with_stuff<W, T, E, L, Wp>(
    world: &W,
    ctx: ContextWrapper<T>,
) -> Option<ContextWrapper<T>>
where
    W: World<Exit = E, Location = L, Warp = Wp>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit<Context = T> + Accessible<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId>,
{
    let spot_map = access(world, ctx);
    if let Some((_, ctx)) = spot_map
        .into_iter()
        .filter(|(s, ctx)| spot_has_locations_or_actions(world, ctx.get(), *s))
        .min_by_key(|(_, c)| c.elapsed())
    {
        Some(ctx)
    } else {
        None
    }
}

pub fn do_and_grab_all<W, T, L, E>(world: &W, ctx: &mut ContextWrapper<T>)
where
    W: World<Exit = E, Location = L>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit<Context = T> + Accessible<Context = T>,
{
    for act in world.get_spot_actions(ctx.get().position()) {
        if act.can_access(ctx.get()) {
            ctx.activate(act);
        }
    }
    let (locs, exit) = visitable_locations(world, ctx.get());
    for loc in locs {
        ctx.visit(world, loc);
    }

    if let Some((l, e)) = exit {
        let exit = world.get_exit(e);
        let loc = world.get_location(l);
        ctx.visit_exit(world, loc, exit);
    }
}

pub fn greedy_search<W, T, L, E>(world: &W, ctx: &ContextWrapper<T>) -> Option<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    let mut ctx = ctx.clone();
    while !world.won(ctx.get()) {
        if let Some(c) = nearest_spot_with_stuff(world, ctx) {
            ctx = c;
        } else {
            return None;
        }
        do_and_grab_all(world, &mut ctx);
    }
    Some(ctx)
}
