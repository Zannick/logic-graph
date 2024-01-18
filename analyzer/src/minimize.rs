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
    for hist in wonctx.recent_history() {
        match hist {
            History::G(_, loc_id) => {
                set[*loc_id] = true;
            }
            History::H(_, exit_id) => {
                let ex = world.get_exit(*exit_id);
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

/// Attempts to minimize a route by skipping item locations
/// and returning a new start state.
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
    for hist in wonctx.recent_history() {
        match hist {
            History::G(_, loc_id) => {
                ctx.skip(*loc_id);
                // TODO: If this location can be replaced by an action, e.g. collect rupees,
                // then it will be dropped, and if the action is slower, we fail to minimize
                // to a shorter playthrough.
                if can_win(world, &ctx, wonctx.elapsed()).is_err() {
                    ctx.reset(*loc_id);
                }
            }
            History::H(_, exit_id) => {
                let ex = world.get_exit(*exit_id);
                if let Some(loc_id) = ex.loc_id() {
                    ctx.skip(*loc_id);
                    if can_win(world, &ctx, wonctx.elapsed()).is_err() {
                        ctx.reset(*loc_id);
                    }
                }
            }
            _ => (),
        }
    }

    ContextWrapper::new(ctx)
}

/// Attempts to create better solutions by removing items collected from a route.
/// Returns only one such possibility.
pub fn pinpoint_minimize<W, T, L, E>(
    world: &W,
    startctx: &T,
    history: &[HistoryAlias<T>],
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let mut ctx = ContextWrapper::new(startctx.clone());
    let mut best = None;
    'main: for i in 0..history.len() {
        let step = history[i];
        if matches!(step, History::G(_, _)) {
            let mut sub = ctx.clone();
            for j in (i + 1)..history.len() {
                if sub.can_replay(world, history[j]) {
                    sub.replay(world, history[j]);
                } else {
                    ctx.assert_and_replay(world, step);
                    continue 'main;
                }
            }
            // successfully applied all, but we still have to check whether we won
            if world.won(sub.get()) {
                best = Some(sub);
                // now we'll skip this step and continue with ctx
                continue 'main;
            } else {
                ctx.assert_and_replay(world, step);
                continue 'main;
            }
        } else {
            ctx.assert_and_replay(world, step);
        }
    }

    best
}
