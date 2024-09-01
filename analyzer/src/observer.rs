use crate::context::{history_to_full_series, Ctx, History, HistoryAlias};
use crate::matchertrie::*;
use crate::solutions::{Solution, SolutionSuffix};
use crate::world::*;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

/// An Observer tracks a set of observations.
///
/// Observations must be tracked in reverse. We handle this by providing `observe_` APIs that record
/// that an observation is being made, and an `apply_observations` function that applies said observations
/// in reverse order. This way observations can be still recorded in the order they occur.
pub trait Observer: Debug + Default {
    type Ctx: Ctx;

    /// Creates a new observation set from a winning state.
    fn from_victory_state(won: &Self::Ctx, world: &<Self::Ctx as Ctx>::World) -> Self;

    /// Records that we know whether this location is visited.
    fn observe_visited(
        &mut self,
        loc_id: <<<Self::Ctx as Ctx>::World as World>::Location as Location>::LocId,
    );

    /// Records that we know an item was added.
    fn observe_item(&mut self, item: <Self::Ctx as Ctx>::ItemId);

    /// Applies the most recent set of observation updates (from `observe_` functions) in reverse order.
    fn apply_observations(&mut self);

    /// Exports a list of individual property observations for consumption by the matcher trie.
    fn to_vec(&self, ctx: &Self::Ctx) -> Vec<<Self::Ctx as Observable>::PropertyObservation>;
}

pub trait TrieMatcher<ValueType: Clone + Eq + Hash>:
    MatcherDispatch<ValueType, Node = Node<Self, ValueType>> + Default + Send + Sync + 'static
{
}
impl<TM, ValueType> TrieMatcher<ValueType> for TM
where
    TM: MatcherDispatch<ValueType, Node = Node<Self, ValueType>> + Default + Send + Sync + 'static,
    ValueType: Clone + Eq + Hash,
{
}

// This is here to allow benchmarking without a SolutionCollector.
/// Records a full solution's observations into the solve trie.
///
/// Every state that has visited at least |min_relevant| locations is recorded in the trie,
/// except for the winning state.
pub fn record_observations<W, T, TM>(
    startctx: &T,
    world: &W,
    solution: Arc<Solution<T>>,
    min_relevant: usize,
    solve_trie: &MatcherTrie<TM, SolutionSuffix<T>>,
) where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
    TM: TrieMatcher<SolutionSuffix<T>, Struct = T>,
{
    let full_history = history_to_full_series(startctx, world, solution.history.iter().copied());
    // The history entries are the steps "in between" the states in full_history, so we should have
    // one more state than history steps.
    assert!(full_history.len() == solution.history.len() + 1);
    let prev = full_history.last().unwrap();
    let mut solve = <T::Observer as Observer>::from_victory_state(prev, world);

    let mut pcount = 0;
    let skippable = if min_relevant > 1 {
        solution
            .history
            .iter()
            .position(|h| match h {
                History::G(..) | History::V(..) => {
                    pcount += 1;
                    pcount == min_relevant
                }
                _ => false,
            })
            .unwrap_or(1)
    } else {
        solution
            .history
            .iter()
            .position(|h| matches!(h, History::G(..) | History::V(..)))
            .unwrap_or(1)
    };

    for (idx, (step, state)) in solution
        .history
        .iter()
        .zip(full_history.iter())
        .enumerate()
        .skip(skippable)
        .rev()
    {
        // Basic process of iterating backwards:
        // 1. Observe the history step requirements/effects itself.
        state.observe_replay(world, *step, &mut solve);

        // 2. Apply the observations in reverse order.
        solve.apply_observations();

        // 3. Insert the new observation list.
        solve_trie.insert(solve.to_vec(state), SolutionSuffix(solution.clone(), idx));
    }
}

/// Records a non-winning step sequence into the given trie.
///
/// This does not need to start from nothing, but the solution provided must be applicable from the starting state.
pub fn record_short_observations<W, T, TM>(
    startctx: &T,
    world: &W,
    solution: Arc<Solution<T>>,
    short_trie: &MatcherTrie<TM, SolutionSuffix<T>>,
) where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
    TM: TrieMatcher<SolutionSuffix<T>, Struct = T>,
{
    let full_history = history_to_full_series(startctx, world, solution.history.iter().copied());
    // The history entries are the steps "in between" the states in full_history, so we should have
    // one more state than history steps.
    assert!(full_history.len() == solution.history.len() + 1);
    let mut short_obs = T::Observer::default();

    for (idx, (step, state)) in solution
        .history
        .iter()
        .zip(full_history.iter())
        .enumerate()
        .rev()
    {
        // Basic process of iterating backwards:
        // 1. Observe the history step requirements/effects itself.
        state.observe_replay(world, *step, &mut short_obs);

        // 2. Apply the observations in reverse order.
        short_obs.apply_observations();

        // 3. Insert the new observation list.
        short_trie.insert(
            short_obs.to_vec(state),
            SolutionSuffix(solution.clone(), idx),
        );
    }
}

/// Returns a route's observation sets for the item acquisition steps.
///
/// These are in the same order as collection_history functions, but will need to be zipped separately.
pub fn collection_observations<W, T>(
    startctx: &T,
    world: &W,
    history: &[HistoryAlias<T>],
) -> Vec<Vec<T::PropertyObservation>>
where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
{
    let full_history = history_to_full_series(startctx, world, history.iter().copied());
    // The history entries are the steps "in between" the states in full_history, so we should have
    // one more state than history steps.
    assert!(full_history.len() == history.len() + 1);
    let prev = full_history.last().unwrap();
    let mut solve = <T::Observer as Observer>::from_victory_state(prev, world);

    let mut obs_list = Vec::new();

    for (step, state) in history.iter().zip(full_history.iter()).rev() {
        // Basic process of iterating backwards:
        // 1. Observe the history step requirements/effects itself.
        state.observe_replay(world, *step, &mut solve);

        // 2. Apply the observations in reverse order.
        solve.apply_observations();

        // 3. Insert the new observation list.
        match step {
            History::G(..) | History::V(..) => obs_list.push(solve.to_vec(state)),
            History::A(act_id) if W::action_has_visit(*act_id) => {
                obs_list.push(solve.to_vec(state))
            }
            _ => (),
        }
    }
    obs_list.reverse();
    obs_list
}

/// Outputs (in reverse order) a route's steps and observations.
pub fn debug_observations<W, T>(
    startctx: &T,
    world: &W,
    solution: Arc<Solution<T>>,
    min_relevant: usize,
) where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
{
    let full_history = history_to_full_series(startctx, world, solution.history.iter().copied());
    // The history entries are the steps "in between" the states in full_history, so we should have
    // one more state than history steps.
    assert!(full_history.len() == solution.history.len() + 1);
    let prev = full_history.last().unwrap();
    let mut solve = <T::Observer as Observer>::from_victory_state(prev, world);

    let mut pcount = 0;
    let skippable = if min_relevant > 1 {
        solution
            .history
            .iter()
            .position(|h| match h {
                History::G(..) | History::V(..) => {
                    pcount += 1;
                    pcount == min_relevant
                }
                _ => false,
            })
            .unwrap_or(1)
    } else {
        solution
            .history
            .iter()
            .position(|h| matches!(h, History::G(..) | History::V(..)))
            .unwrap_or(1)
    };

    for (idx, (step, state)) in solution
        .history
        .iter()
        .zip(full_history.iter())
        .enumerate()
        .skip(skippable)
        .rev()
    {
        // Basic process of iterating backwards:
        // 1. Observe the history step requirements/effects itself.
        state.observe_replay(world, *step, &mut solve);

        // 2. Apply the observations in reverse order.
        solve.apply_observations();

        // 3. Output what we have.
        println!("{}. {}\n{:?}\n", idx, step, solve.to_vec(state));
    }
}
