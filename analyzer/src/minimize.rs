use crate::context::*;
use crate::matchertrie::MatcherTrie;
use crate::observer::record_observations;
use crate::observer::Observer;
use crate::solutions::Solution;
use crate::world::*;
use std::collections::VecDeque;
use std::sync::Arc;

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
    record_observations(startctx, world, solution.clone(), 0, &mut trie);
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
    let mut replay = ContextWrapper::new(startctx.clone());
    let mut index = 0;
    let mut best_elapsed = best_solution.elapsed;
    let mut best = None;
    while index < best_solution.history.len() {
        replay.assert_and_replay(world, best_solution.history[index]);
        index += 1;
        let mut queue = VecDeque::from(trie.lookup(replay.get()));
        'q: while let Some(suffix) = queue.pop_front() {
            if suffix.suffix() == &best_solution.history[index..] {
                continue;
            }
            let mut r2 = replay.clone();
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
                best_solution = suffix.0.clone();
                index = suffix.1;
                best_elapsed = r2.elapsed();
                best = Some(r2);
            }
        }
    }
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
            if r2.elapsed() >= best_elapsed {
                continue 'q;
            }
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
