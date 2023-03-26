use enum_map::EnumMap;

use crate::access::*;
use crate::context::*;
use crate::world::*;

pub fn remove_all_unvisited<W, T, L, E>(world: &W, startctx: &T, wonctx: &ContextWrapper<T>) -> T
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let mut ctx = startctx.clone();
    let mut set: EnumMap<E::LocId, bool> = EnumMap::default();
    // Gather locations from the playthrough
    for hist in wonctx.history_rev() {
        match hist {
            History::Get(_, loc_id) => {
                set[loc_id] = true;
            }
            History::MoveGet(_, exit_id) => {
                let ex = world.get_exit(exit_id);
                if let Some(loc_id) = ex.loc_id() {
                    set[*loc_id] = true;
                }
            }
            _ => (),
        }
    }
    let set = set;

    // skip all locations not in the playthrough
    for loc in world.get_all_locations() {
        if set[loc.id()] {
            continue;
        }
        ctx.skip(loc.id());
    }
    ctx
}

pub fn minimize<W, T, L, E>(
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
    let mut ctx = remove_all_unvisited(world, startctx, wonctx);

    // skip remaining visited locations from last to first
    for hist in wonctx.history_rev() {
        match hist {
            History::Get(_, loc_id) => {
                ctx.skip(loc_id);
                // TODO: If this location can be replaced by an action, e.g. collect rupees,
                // then it will be dropped, and if the action is slower, we fail to minimize
                // to a shorter playthrough.
                if !can_win(world, &ctx, wonctx.elapsed()).is_ok() {
                    ctx.reset(loc_id);
                }
            }
            History::MoveGet(_, exit_id) => {
                let ex = world.get_exit(exit_id);
                if let Some(loc_id) = ex.loc_id() {
                    ctx.skip(*loc_id);
                    if !can_win(world, &ctx, wonctx.elapsed()).is_ok() {
                        ctx.reset(*loc_id);
                    }
                }
            }
            _ => (),
        }
    }

    ContextWrapper::new(ctx)
}
