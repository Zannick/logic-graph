use crate::access::{
    access_action_after_actions, access_action_after_actions_with_req,
    access_location_after_actions, access_location_after_actions_with_req,
};
use crate::context::*;
use crate::matchertrie::MatcherTrie;
use crate::new_hashmap;
use crate::observer::{collection_observations, record_observations, TrieMatcher};
use crate::solutions::{Solution, SolutionSuffix};
use crate::steiner::{EdgeId, NodeId, ShortestPaths};
use crate::world::*;
use crate::CommonHasher;
use std::collections::{HashMap, VecDeque};
use std::ops::RangeInclusive;
use std::sync::Arc;

/// Attempts to create better solutions by removing sections of the route
/// based on observations.
pub fn pinpoint_minimize<W, T, L, TM>(
    world: &W,
    startctx: &T,
    solution: Arc<Solution<T>>,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    W::Exit: Exit<Context = T, Currency = L::Currency>,
    TM: TrieMatcher<SolutionSuffix<T>, Struct = T>,
{
    let mut trie = MatcherTrie::<TM, SolutionSuffix<T>>::default();
    record_observations(startctx, world, solution.clone(), 0, &mut trie);
    trie_minimize(world, startctx, solution, &trie)
}

/// Produces a map of spots to a list of indices where the route moves into that spot.
fn get_spot_index_map<W, T, E>(
    world: &W,
    startctx: &T,
    solution: Arc<Solution<T>>,
) -> HashMap<E::SpotId, VecDeque<usize>, CommonHasher>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T>,
    W::Location: Location<Context = T, Currency = E::Currency>,
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
pub fn mutate_spot_revisits<W, T, L>(
    world: &W,
    startctx: &T,
    solution: Arc<Solution<T>>,
) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    W::Exit: Exit<Context = T, Currency = L::Currency>,
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

trait RangeAndStepTuple<T: Ctx> {
    fn range(&self) -> &RangeInclusive<usize>;
    fn step(&self) -> &HistoryAlias<T>;
}

impl<T: Ctx> RangeAndStepTuple<T> for (RangeInclusive<usize>, HistoryAlias<T>) {
    fn range(&self) -> &RangeInclusive<usize> {
        &self.0
    }
    fn step(&self) -> &HistoryAlias<T> {
        &self.1
    }
}

impl<T: Ctx> RangeAndStepTuple<T> for (RangeInclusive<usize>, HistoryAlias<T>, usize) {
    fn range(&self) -> &RangeInclusive<usize> {
        &self.0
    }
    fn step(&self) -> &HistoryAlias<T> {
        &self.1
    }
}

fn rediscover_routes<'a, W, T, L, RT>(
    world: &W,
    mut rreplay: ContextWrapper<T>,
    iter: impl Iterator<Item = &'a RT>,
    max_time: u32,
    max_depth: usize,
    max_states: usize,
    history: &Vec<HistoryAlias<T>>,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    RT: 'a + RangeAndStepTuple<T>,
{
    for tuple in iter {
        let range = tuple.range();
        let step = tuple.step();
        // Attempt to reuse the same direct path if possible.
        let mut ctx = rreplay.clone();
        if ctx.maybe_replay_all(world, &history[range.clone()]) && ctx.elapsed() <= max_time {
            rreplay = ctx;
            continue;
        }
        rreplay = match step {
            History::A(act_id) => access_action_after_actions(
                world,
                rreplay,
                *act_id,
                max_time,
                max_depth,
                max_states,
                shortest_paths,
            ),
            History::G(.., loc_id) => access_location_after_actions(
                world,
                rreplay,
                *loc_id,
                max_time,
                max_depth,
                max_states,
                shortest_paths,
            ),
            _ => return None,
        }
        .ok()?;
    }
    Some(rreplay)
}

fn rediscover_wrapped<'a, W, T, L, RT>(
    world: &W,
    rreplay: Option<ContextWrapper<T>>,
    iter: impl Iterator<Item = &'a RT>,
    max_time: u32,
    max_depth: usize,
    max_states: usize,
    history: &Vec<HistoryAlias<T>>,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    RT: 'a + RangeAndStepTuple<T>,
{
    if let Some(rreplay) = rreplay {
        rediscover_routes(
            world,
            rreplay,
            iter,
            max_time,
            max_depth,
            max_states,
            history,
            shortest_paths,
        )
    } else {
        None
    }
}

pub fn mutate_collection_steps<W, T>(
    world: &W,
    startctx: &T,
    max_time: u32,
    max_depth: usize,
    max_states: usize,
    solution: Arc<Solution<T>>,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Option<ContextWrapper<T>>
where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
    W::Exit: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
{
    // Restrict max time to being strictly less than the given solution, since we'll only return if we improve anyway.
    let max_time = std::cmp::min(max_time, solution.elapsed.saturating_sub(1));
    // [(history range inclusive of the collection step, history step, community)]
    // to recreate the state just before this step, we would replay [..index] (i.e. exclusive)
    let collection_hist: Vec<_> =
        collection_history_with_range_info::<T, _>(solution.history.iter().copied())
            .map(|(r, h)| {
                (
                    r,
                    h,
                    match h {
                        History::G(_, loc_id) | History::V(_, loc_id, _) => {
                            W::location_community(loc_id)
                        }
                        History::A(act_id) => W::action_community(act_id),
                        _ => 0,
                    },
                )
            })
            .collect();
    let mut replay = ContextWrapper::new(startctx.clone());

    // for i, A in collection history[..-1]
    // find the first B after A not in the same community
    // for every C after A in the same community
    // if C is before B, try reordering:
    // 1. just A: (A..C)AC (if A is not right before C)
    // 2. just A after C: (A..C]A (if B is right after C)
    // otherwise, try reordering:
    // 3. just A: (A..B)[B..C)AC
    // 4. all of A's community: [B..C)[A..B)C
    // how many reordering attempts is this?
    // a contiguous clique of size k will see O(k^2) (k^2-3k+2)/2 rearrangements in 1, k-1 in 2
    // 2 discontinuous cliques of size m and n will see 2n rearrangements in 3+4 (plus the m^2 and n^2 in 1+2)
    // Let's remove the n^2 factor of rearranging within a clique, let search do that.
    'next_a: for (coll_ai, (range_a, _, comm)) in collection_hist[..collection_hist.len() - 1]
        .iter()
        .enumerate()
    {
        let mut reorder_just_a = Some(replay.clone());
        // Collect A on the replay *after* we've checkpointed for the reordering attempts.
        assert!(
            replay.maybe_replay_all(world, &solution.history[range_a.clone()]),
            "Could not replay base solution history range {:?}",
            range_a,
        );

        // Skip trying to reorder if this step doesn't have a community.
        if *comm == 0 {
            continue;
        }
        let Some(mut coll_bi) = collection_hist[coll_ai + 1..]
            .iter()
            .position(|(.., bcomm)| bcomm != comm)
        else {
            // also skip if we don't find anything outside the community
            continue;
        };

        let mut reorder_full = reorder_just_a.clone();
        // index is 0-based from slice start
        coll_bi += coll_ai + 1;

        let mut cprev_justa = coll_ai + 1;
        let mut cprev_full = coll_bi;

        // For just 3+4 above, we can start at B + 1.
        for (mut coll_ci, (.., ccomm)) in collection_hist[coll_bi + 1..].iter().enumerate() {
            // index is 0-based from slice start
            coll_ci += coll_bi + 1;
            if ccomm != comm {
                continue;
            }
            reorder_just_a = rediscover_wrapped(
                world,
                reorder_just_a,
                collection_hist[cprev_justa..coll_ci].iter(),
                max_time,
                max_depth,
                max_states,
                &solution.history,
                shortest_paths,
            );
            reorder_full = rediscover_wrapped(
                world,
                reorder_full,
                collection_hist[cprev_full..coll_ci].iter(),
                max_time,
                max_depth,
                max_states,
                &solution.history,
                shortest_paths,
            );
            cprev_justa = coll_ci;
            cprev_full = coll_ci;
            // early exit if replays already broke.
            if matches!((&reorder_just_a, &reorder_full), (&None, &None)) {
                continue 'next_a;
            }

            if let Some(reorder_a) = reorder_just_a.clone() {
                if let Some(reordered) = rediscover_routes(
                    world,
                    reorder_a,
                    collection_hist[coll_ai..=coll_ai]
                        .iter()
                        .chain(&collection_hist[coll_ci..]),
                    max_time,
                    max_depth,
                    max_states,
                    &solution.history,
                    shortest_paths,
                ) {
                    if world.won(reordered.get()) {
                        return Some(reordered);
                    }
                }
            }
            if let Some(reorder_full) = reorder_full.clone() {
                if let Some(reordered) = rediscover_routes(
                    world,
                    reorder_full.clone(),
                    collection_hist[coll_ai..coll_bi]
                        .iter()
                        .chain(&collection_hist[coll_ci..]),
                    max_time,
                    max_depth,
                    max_states,
                    &solution.history,
                    shortest_paths,
                ) {
                    if world.won(reordered.get()) {
                        return Some(reordered);
                    }
                }
                // Just after C
                if let Some(reordered) = rediscover_routes(
                    world,
                    reorder_full,
                    collection_hist[coll_ci..=coll_ci]
                        .iter()
                        .chain(&collection_hist[coll_ai..coll_bi])
                        .chain(&collection_hist[coll_ci + 1..]),
                    max_time,
                    max_depth,
                    max_states,
                    &solution.history,
                    shortest_paths,
                ) {
                    if world.won(reordered.get()) {
                        return Some(reordered);
                    }
                }
            }
        }

        // If we've managed to reroute almost everything, then we can try the last bit, in which case
        // A was unnecessary
        if let Some(all_but_a) = reorder_just_a {
            if let Some(reordered) = rediscover_routes(
                world,
                all_but_a,
                collection_hist[cprev_justa..].iter(),
                max_time,
                max_depth,
                max_states,
                &solution.history,
                shortest_paths,
            ) {
                if world.won(reordered.get()) {
                    return Some(reordered);
                }
            }
        }
        if let Some(all_but_comm) = reorder_full {
            if let Some(reordered) = rediscover_routes(
                world,
                all_but_comm,
                collection_hist[cprev_full..].iter(),
                max_time,
                max_depth,
                max_states,
                &solution.history,
                shortest_paths,
            ) {
                if world.won(reordered.get()) {
                    return Some(reordered);
                }
            }
        }
    }

    None
}

/// Mutate routes by replacing collections with other locations but same canon
pub fn mutate_canon_locations<W, T, L>(
    world: &W,
    startctx: &T,
    mut max_time: u32,
    max_depth: usize,
    max_states: usize,
    solution: Arc<Solution<T>>,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    W::Exit: Exit<Context = T, Currency = L::Currency>,
{
    // Restrict max time to being strictly less than the given solution, since we'll only return if we improve anyway.
    max_time = std::cmp::min(max_time, solution.elapsed.saturating_sub(1));
    // [(history index, history step)]
    // to recreate the state just before this step, we would replay [..index] (i.e. exclusive)
    let collection_hist: Vec<_> =
        collection_history_with_range_info::<T, _>(solution.history.iter().copied()).collect();
    let mut replay = ContextWrapper::new(startctx.clone());
    let mut best = None;

    let mut count = 0;
    for (coll, (_, step)) in collection_hist.iter().enumerate() {
        // Clone first, then advance the replay.
        let replace_this_collection = replay.clone();

        let old_loc_id = match step {
            History::G(_, loc_id) | History::V(_, loc_id, _) => loc_id,
            _ => continue,
        };

        let mut found = false;
        // For every other loc id with this canon id
        for loc_id in W::get_canon_location_ids(world.get_location(*old_loc_id).canon_id()) {
            if loc_id == old_loc_id {
                continue;
            }
            // Search for a way to that alternative location
            if let Ok(replaced) = access_location_after_actions(
                world,
                replace_this_collection.clone(),
                *loc_id,
                max_time,
                max_depth,
                max_states,
                shortest_paths,
            ) {
                // And if there is such a way, rediscover routes to the same locations in the rest of the route
                if let Some(reordered) = rediscover_routes(
                    world,
                    replaced.clone(),
                    collection_hist[coll + 1..].iter(),
                    max_time,
                    max_depth,
                    max_states,
                    &solution.history,
                    shortest_paths,
                ) {
                    if world.won(reordered.get()) {
                        max_time = reordered.elapsed().saturating_sub(1);
                        best = Some(reordered);
                        replay = replaced;
                        found = true;
                    }
                }
            }
        }
        if found {
            count += 1;
        } else {
            let Some(rreplay) = rediscover_routes(
                world,
                replay,
                collection_hist[coll..=coll].iter(),
                max_time,
                max_depth,
                max_states,
                &solution.history,
                shortest_paths,
            ) else {
                log::debug!("Replaced {} location(s) with canon equivalents (and then failed to replay further)", count);
                return best;
            };
            replay = rreplay;
        }
    }
    log::debug!("Replaced {} location(s) with canon equivalents", count);

    best
}

/// Mutate routes between collections by finding a greedy path to the next
pub fn mutate_greedy_collections<W, T, L>(
    world: &W,
    startctx: &T,
    _max_time: u32,
    max_depth: usize,
    max_states: usize,
    solution: Arc<Solution<T>>,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    W::Exit: Exit<Context = T, Currency = L::Currency>,
{
    // [(history index, history step)]
    // to recreate the state just before this step, we would replay [..index] (i.e. exclusive)
    let collection_hist: Vec<_> =
        collection_history_with_range_info::<T, _>(solution.history.iter().copied()).collect();
    let obs_list = collection_observations(startctx, world, &solution.history);
    let mut replay = ContextWrapper::new(startctx.clone());

    for ((range_a, step), observations) in collection_hist.iter().zip(obs_list.iter()) {
        // Clone first, then advance the replay and the attempt.
        let attempt = replay.clone();
        assert!(
            replay.maybe_replay_all(world, &solution.history[range_a.clone()]),
            "Could not replay base solution history range {:?}",
            range_a,
        );
        let Ok(attempt) = (match step {
            History::A(act_id) => access_action_after_actions_with_req(
                world,
                attempt,
                *act_id,
                replay.elapsed(),
                max_depth,
                max_states,
                |c| c.matches_all(observations),
                shortest_paths,
            ),
            History::G(.., loc_id) | History::V(_, loc_id, _) => {
                access_location_after_actions_with_req(
                    world,
                    attempt,
                    *loc_id,
                    replay.elapsed(),
                    max_depth,
                    max_states,
                    |c| {
                        c.position() == world.get_location_spot(*loc_id)
                            && world.get_location(*loc_id).can_access(c, world)
                            && c.matches_all(observations)
                    },
                    shortest_paths,
                )
            }
            _ => continue,
        }) else {
            continue;
        };
        // If the attempt isn't faster, we don't care. Usually it should find the same route and be equal.
        if attempt.elapsed() >= replay.elapsed() {
            continue;
        }

        // If the attempt state is the same as the replay, then we've improved the result already,
        // so we continue with the new result.
        if attempt.get() == replay.get() {
            replay = attempt;
            continue;
        }

        // Even if it's not the same, we matched the observations just before the collection, and performed
        // the same collection, so we should be able to replay the rest.
        if let Ok(best) = attempt.try_replay_all(world, &solution.history[range_a.end() + 1..]) {
            if best.elapsed() < solution.elapsed && world.won(best.get()) {
                return Some(best);
            }
        }
    }
    // If we didn't find a better route that has a state deviation (i.e. 2 or 3), we can use the replay if it's an improvement
    // (i.e. from 1).
    if replay.elapsed() < solution.elapsed {
        Some(replay)
    } else {
        None
    }
}

/// Use a matcher trie to minimize a solution
pub fn trie_minimize<W, T, L, TM>(
    world: &W,
    startctx: &T,
    mut best_solution: Arc<Solution<T>>,
    trie: &MatcherTrie<TM, SolutionSuffix<T>>,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    W::Exit: Exit<Context = T, Currency = L::Currency>,
    TM: TrieMatcher<SolutionSuffix<T>, Struct = T>,
{
    let mut replay = ContextWrapper::new(startctx.clone());
    let mut index = 0;
    let mut best_elapsed = best_solution.elapsed;
    let mut best = None;
    while index < best_solution.history.len() {
        let mut queue = VecDeque::from(trie.lookup(replay.get()));
        'q: while let Some(suffix) = queue.pop_front() {
            // Don't bother with the attempt if it's the same as our current best.
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
        replay.assert_and_replay(world, best_solution.history[index]);
        index += 1;
    }
    best
}

pub fn trie_search<W, T, L, TM>(
    world: &W,
    ctx: &ContextWrapper<T>,
    max_time: u32,
    trie: &MatcherTrie<TM, SolutionSuffix<T>>,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
    W::Exit: Exit<Context = T, Currency = L::Currency>,
    TM: TrieMatcher<SolutionSuffix<T>, Struct = T>,
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
