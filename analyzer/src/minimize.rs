use crate::access::{access_action_after_actions, access_location_after_actions};
use crate::context::*;
use crate::matchertrie::MatcherTrie;
use crate::new_hashmap;
use crate::observer::{record_observations, Observer};
use crate::solutions::Solution;
use crate::steiner::{EdgeId, NodeId, ShortestPaths};
use crate::world::*;
use crate::CommonHasher;
use std::collections::{HashMap, VecDeque};
use std::ops::RangeInclusive;
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

fn rediscover_routes<'a, W, T, L, I>(
    world: &W,
    mut rreplay: ContextWrapper<T>,
    iter: I,
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
    I: Iterator<Item = &'a (RangeInclusive<usize>, HistoryAlias<T>, usize)>,
{
    for (range, step, _) in iter {
        // Attempt to reuse the same direct path if possible.
        let mut ctx = rreplay.clone();
        if ctx.maybe_replay_all(world, &history[range.clone()]) {
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
            History::H(.., exit_id) => access_location_after_actions(
                world,
                rreplay,
                world.get_exit(*exit_id).loc_id().unwrap(),
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

fn rediscover_wrapped<'a, W, T, L, I>(
    world: &W,
    rreplay: Option<ContextWrapper<T>>,
    iter: I,
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
    I: Iterator<Item = &'a (RangeInclusive<usize>, HistoryAlias<T>, usize)>,
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

pub fn mutate_collection_steps<W, T, L, E>(
    world: &W,
    startctx: &T,
    max_time: u32,
    max_depth: usize,
    max_states: usize,
    solution: Arc<Solution<T>>,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Option<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId, Context = T, Currency = E::Currency>,
    E: Exit<Context = T>,
{
    // [(history index, history step, community)]
    // to recreate the state just before this step, we would replay [..index] (i.e. exclusive)
    let collection_hist: Vec<_> =
        collection_history_with_range_info::<T, W, L, _>(solution.history.iter().copied())
            .map(|(r, h)| {
                (
                    r,
                    h,
                    match h {
                        History::G(_, loc_id) => W::location_community(loc_id),
                        History::H(_, exit_id) => W::exit_community(exit_id),
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
    for (coll_ai, (range_a, _, comm)) in collection_hist[..collection_hist.len() - 1]
        .iter()
        .enumerate()
    {
        assert!(
            replay.maybe_replay_all(world, &solution.history[range_a.clone()]),
            "Could not replay base solution history range {:?}",
            range_a,
        );
        if *comm == 0 {
            continue;
        }
        let Some(mut coll_bi) = collection_hist[coll_ai + 1..]
            .iter()
            .position(|(.., bcomm)| bcomm != comm)
        else {
            // ignore if we don't find anything outside the community
            continue;
        };
        // index is 0-based from slice start
        coll_bi += coll_ai + 1;

        let mut reorder_just_a = Some(replay.clone());
        let mut reorder_full = Some(replay.clone());
        let mut cprev_justa = coll_ai + 1;
        let mut cprev_full = coll_bi;

        // For just 3+4 above, we can start at B + 1.
        for (mut coll_ci, (.., ccomm)) in
            collection_hist[coll_bi + 1..].iter().enumerate()
        {
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
                break;
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
                    if reordered.elapsed() < solution.elapsed && world.won(reordered.get()) {
                        return Some(reordered);
                    }
                }
            }
            if let Some(reorder_full) = reorder_full.clone() {
                if let Some(reordered) = rediscover_routes(
                    world,
                    reorder_full,
                    collection_hist[coll_ai..coll_bi]
                        .iter()
                        .chain(&collection_hist[coll_ci..]),
                    max_time,
                    max_depth,
                    max_states,
                    &solution.history,
                    shortest_paths,
                ) {
                    if reordered.elapsed() < solution.elapsed && world.won(reordered.get()) {
                        return Some(reordered);
                    }
                }
            }
        }
    }

    None
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
