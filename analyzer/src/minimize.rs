use enum_map::EnumMap;

use crate::access::*;
use crate::context::*;
use crate::matchertrie::MatcherTrie;
use crate::observer::record_observations;
use crate::observer::Observer;
use crate::solutions::Solution;
use crate::world::*;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

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

/// Attempts to create better solutions by removing sections of the route.
pub fn pinpoint_minimize<W, T, L, E>(
    world: &W,
    startctx: &T,
    solution: Arc<Solution<T>>,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let mut trie = MatcherTrie::<<T::Observer as Observer>::Matcher>::default();
    record_observations(startctx, world, solution.clone(), 0, None, &mut trie);
    trie_minimize(world, startctx, solution, &trie)
}

/// Use a matcher trie to minimize a solution
pub fn trie_minimize<W, T, L, E>(
    world: &W,
    startctx: &T,
    mut best_solution: Arc<Solution<T>>,
    trie: &MatcherTrie<<T::Observer as Observer>::Matcher>,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let mut valid = 0;
    let mut invalid = 0;
    let mut equiv = 0;
    let mut replay = ContextWrapper::new(startctx.clone());
    let mut index = 0;
    let start = Instant::now();
    let mut best_elapsed = best_solution.elapsed;
    let mut best = None;
    while index < best_solution.history.len() {
        replay.assert_and_replay(world, best_solution.history[index]);
        index += 1;
        let mut queue = VecDeque::from(trie.lookup(replay.get()));
        'q: while let Some(suffix) = queue.pop_front() {
            if suffix.suffix() == &best_solution.history[index..] {
                equiv += 1;
                continue;
            }
            let mut r2 = replay.clone();
            for step in suffix.suffix() {
                if !r2.can_replay(world, *step) {
                    invalid += 1;
                    continue 'q;
                }
                r2.replay(world, *step);
            }
            if !world.won(r2.get()) {
                invalid += 1;
                continue 'q;
            }

            valid += 1;
            if r2.elapsed() < best_elapsed {
                best_solution = suffix.0.clone();
                index = suffix.1;
                best_elapsed = r2.elapsed();
                best = Some(r2);
            }
        }
    }
    log::info!(
        "Trie minimize took {:?} and found {} equivalent, {} valid, and {} invalid derivative paths.",
        start.elapsed(), equiv, valid, invalid,
    );
    best
}

pub fn trie_search<W, T, L, E>(
    world: &W,
    ctx: &ContextWrapper<T>,
    max_time: u32,
    trie: &MatcherTrie<<T::Observer as Observer>::Matcher>,
) -> Option<ContextWrapper<T>> where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let mut queue = VecDeque::from(trie.lookup(ctx.get()));
    let mut best = None;
    let mut best_elapsed = max_time;
    'q: while let Some(suffix) = queue.pop_front() {
        let mut r2 = ctx.clone();
        for step in suffix.suffix() {
            if !r2.can_replay(world, *step) {
                continue 'q;
            }
            r2.replay(world, *step);
        }
        if !world.won(r2.get()) {
            continue 'q;
        }

        if r2.elapsed() < best_elapsed {
            best_elapsed = r2.elapsed();
            best = Some(r2);
        }
    }

    best
}
