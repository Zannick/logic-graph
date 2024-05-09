use crate::context::*;
use crate::matchertrie::MatcherTrie;
use crate::new_hashmap;
use crate::observer::record_observations;
use crate::observer::Observer;
use crate::solutions::Solution;
use crate::world::*;
use std::collections::VecDeque;
use std::sync::Arc;

/// Attempts to create better solutions by removing sections of the route
/// based on observations.
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

/// Attempts to create better solutions by removing sections of the route
/// that revisit spots.
pub fn spot_revisit_minimize<W, T, L, E>(
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
    let mut replay = ContextWrapper::new(startctx.clone());
    let mut spot_map = new_hashmap();
    // position -> list of step indices where the state is at that position before executing that step
    spot_map.insert(replay.get().position(), VecDeque::from(vec![0]));
    for (index, step) in solution.history.iter().enumerate() {
        replay.assert_and_replay(world, *step);
        if let Some(deq) = spot_map.get_mut(&replay.get().position()) {
            deq.push_back(index + 1);
        } else {
            let mut deq = VecDeque::new();
            deq.push_back(index + 1);
            spot_map.insert(replay.get().position(), deq);
        }
    }
    spot_map.retain(|_, deq| deq.len() > 1);
    if spot_map.is_empty() {
        return None;
    }
    // Indices are unique
    let mut index_map = new_hashmap();
    for (spot, deq) in &spot_map {
        for i in deq {
            index_map.insert(*i, *spot);
        }
    }
    let mut indexes: Vec<usize> = Vec::with_capacity(index_map.len());
    indexes.extend(index_map.keys());
    indexes.sort_unstable();
    indexes.reverse();
    // Iterating through indexes as a "start" index is akin to iterating forward through the replay
    let mut last_index = 0;
    let mut replay = ContextWrapper::new(startctx.clone());
    let mut best = None;
    while let Some(start_index) = indexes.pop() {
        // start_index is the index of a step where the state is at "spot" just before executing
        // to recreate that state, we replay up to but not including start_index
        for step in &solution.history[last_index..start_index] {
            replay.assert_and_replay(world, *step);
        }
        last_index = start_index;
        let spot = index_map
            .get(&start_index)
            .expect("index missing from index map");
        let deq = spot_map
            .get_mut(spot)
            .expect("spot in index map missing from spot map");
        // Since the deqs are ascending, this should always be true at least once.
        while !deq.is_empty() && deq.front() <= Some(&start_index) {
            deq.pop_front();
        }
        for &next_index in deq.iter() {
            assert!(
                next_index > start_index,
                "Expected the spot map values to be in sorted order: start={} ({}), deq={:?}",
                start_index,
                spot,
                deq
            );
            // Don't bother skipping anything that kept us in the same place once.
            if next_index == start_index + 1 {
                continue;
            }
            if let Some(improved) = replay
                .clone()
                .maybe_replay_all(world, &solution.history[next_index..])
            {
                best = Some(improved);
                last_index = next_index;
                log::info!("Cut out a loop! {} to {}", start_index, next_index);
                // Skip some indexes.
                while let Some(index) = indexes.last() {
                    if *index < next_index {
                        indexes.pop();
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }
    best
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
) -> Option<ContextWrapper<T>>
where
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
