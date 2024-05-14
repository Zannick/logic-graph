use crate::context::*;
use crate::matchertrie::MatcherTrie;
use crate::new_hashmap;
use crate::observer::{record_observations, Observer};
use crate::solutions::Solution;
use crate::world::*;
use crate::CommonHasher;
use std::collections::{HashMap, VecDeque};
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

/// Produces a map of spots to a list of indices where the route moves into that spot.
fn get_spot_index_map<W, T, L, E>(
    world: &W,
    startctx: &T,
    solution: Arc<Solution<T>>,
) -> HashMap<E::SpotId, VecDeque<usize>, CommonHasher>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let mut replay = ContextWrapper::new(startctx.clone());
    let mut spot_map = new_hashmap();
    // position -> list of step indices where the state has just moved into that spot
    let mut last_spot = replay.get().position();
    spot_map.insert(replay.get().position(), VecDeque::from(vec![0]));
    for (index, step) in solution.history.iter().enumerate() {
        replay.assert_and_replay(world, *step);
        let pos = replay.get().position();
        // Exclude times when we didn't move.
        if pos == last_spot {
            continue;
        }
        last_spot = pos;
        if let Some(deq) = spot_map.get_mut(&replay.get().position()) {
            deq.push_back(index + 1);
        } else {
            let mut deq = VecDeque::new();
            deq.push_back(index + 1);
            spot_map.insert(replay.get().position(), deq);
        }
    }
    spot_map
}

fn get_index_map_and_list<S>(
    spot_map: &HashMap<S, VecDeque<usize>, CommonHasher>,
) -> (HashMap<usize, S, CommonHasher>, Vec<usize>)
where
    S: Copy,
{
    let mut index_map = new_hashmap();
    for (spot, deq) in spot_map.iter() {
        for i in deq {
            index_map.insert(*i, *spot);
        }
    }
    let mut indexes: Vec<usize> = Vec::with_capacity(index_map.len());
    indexes.extend(index_map.keys());
    indexes.sort_unstable();
    (index_map, indexes)
}

/// Attempts to reorder segments of the solution that are at the same point.
pub fn mutate_spot_revisits<W, T, L, E>(
    world: &W,
    startctx: &T,
    solution: Arc<Solution<T>>,
) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    let mut spot_map = get_spot_index_map(world, startctx, solution.clone());
    spot_map.retain(|_, deq| deq.len() > 2);
    let mut vec = Vec::new();

    // We want to calculate the index->spot map after calling retain so we don't include indexes we don't care about
    // This is mostly to let us iterate forward once through the history for a base replay.
    let (index_map, indexes) = get_index_map_and_list(&spot_map);

    // Iterating through indexes as a "start" index is akin to iterating forward through the replay
    let mut last_index = 0;
    let mut replay = ContextWrapper::new(startctx.clone());
    for start_index in indexes {
        // start_index is the index of a step where the state is at "spot" just before executing.
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
        if deq.len() < 2 {
            continue;
        }

        // Given distinct times visiting a spot A, B, C, we can attempt to swap the order of A->B and B->C.
        let index_b = deq[0];
        let index_c = deq[1];
        // A->B is [start_index..index_b]
        // B->C is [index_b..index_c]
        // C->F is [index_c..]
        let mut swapped = replay.clone();
        if swapped.maybe_replay_all(world, &solution.history[index_b..index_c])
            && swapped.maybe_replay_all(world, &solution.history[start_index..index_b])
            && swapped.maybe_replay_all(world, &solution.history[index_c..])
        {
            vec.push(swapped);
        } else if swapped.recent_history().len() > replay.recent_history().len() {
            vec.push(swapped);
        }
    }

    vec
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
