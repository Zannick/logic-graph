use crate::context::{history_to_full_series, Ctx, History};
use crate::matchertrie::*;
use crate::solutions::{Solution, SolutionSuffix};
use crate::world::*;
use crate::CommonHasher;
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;

/// An Observer tracks a set of observations.
pub trait Observer: Debug {
    type Ctx: Ctx;
    type Matcher: MatcherDispatch<
            Node = Node<Self::Matcher>,
            Struct = Self::Ctx,
            Value = SolutionSuffix<Self::Ctx>,
        > + Default
        + Send
        + Sync
        + 'static;

    /// Creates a new observation set from a winning state.
    fn from_victory_state(won: &Self::Ctx, world: &<Self::Ctx as Ctx>::World) -> Self;

    /// Updates this observation set to mark that we know whether this location is visited or skipped.
    fn observe_visit(
        &mut self,
        loc_id: <<<Self::Ctx as Ctx>::World as World>::Location as Location>::LocId,
    );

    /// Updates this observation set based on any checks that would be made by collect rules.
    fn observe_collect(
        &mut self,
        ctx: &Self::Ctx,
        item_id: <Self::Ctx as Ctx>::ItemId,
        world: &<Self::Ctx as Ctx>::World,
    );

    /// Updates this observation set based on any checks that would be made by on_entry rules.
    fn observe_on_entry(
        &mut self,
        cur: &Self::Ctx,
        dest: <<<Self::Ctx as Ctx>::World as World>::Exit as Exit>::SpotId,
        world: &<Self::Ctx as Ctx>::World,
    );

    /// Updates this observation's bounds based on the difference between two states.
    ///
    /// |from| is the state this observation currently refers to, and |to| is the state we want to refer to next.
    /// (Usually we work backwards, so |to| is the state immediately prior to |from|.)
    fn update(&mut self, from: &Self::Ctx, to: &Self::Ctx);

    /// Exports a list of individual property observations for consumption by the matcher trie.
    fn to_vec(&self, ctx: &Self::Ctx) -> Vec<<Self::Ctx as Observable>::PropertyObservation>;
}

// This is here to allow benchmarking without a SolutionCollector.
/// Records a full solution's observations into the solve trie.
///
/// Every state that has visited at least |min_progress| of the progress_locations is recorded in the trie,
/// except for the winning state.
pub fn record_observations<W, T, L, E, Wp>(
    startctx: &T,
    world: &W,
    solution: Arc<Solution<T>>,
    min_progress: usize,
    progress_locations: &HashSet<L::LocId, CommonHasher>,
    solve_trie: &MatcherTrie<<T::Observer as Observer>::Matcher>,
) where
    W: World<Location = L, Exit = E, Warp = Wp>,
    L: Location<Context = T>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <L as Accessible>::Currency, LocId = L::LocId>,
    Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
{
    let full_history = history_to_full_series(startctx, world, solution.history.iter().copied());
    // The history entries are the steps "in between" the states in full_history, so we should have
    // one more state than history steps.
    assert!(full_history.len() == solution.history.len() + 1);
    let mut prev = full_history.last().unwrap();
    let mut solve = <T::Observer as Observer>::from_victory_state(prev, world);

    let mut pcount = 0;
    let skippable = solution
        .history
        .iter()
        .position(|h| match h {
            History::G(_, loc_id) => {
                if progress_locations.contains(&loc_id) {
                    pcount += 1;
                }
                pcount == min_progress
            }
            History::H(_, exit_id) => {
                let exit = world.get_exit(*exit_id);
                if let Some(loc_id) = exit.loc_id() {
                    if progress_locations.contains(&loc_id) {
                        pcount += 1;
                    }
                }
                pcount == min_progress
            }
            _ => false,
        })
        .unwrap_or(1);

    for (idx, (step, state)) in solution
        .history
        .iter()
        .zip(full_history.iter())
        .enumerate()
        .skip(skippable)
        .rev()
    {
        // Basic process of iterating backwards:
        // 1. Update the existing observations for changes.
        solve.update(prev, state);

        // 2. Observe the history step requirements/effects itself.
        state.observe_replay(world, *step, &mut solve);

        // 3. Insert the new observation list.
        solve_trie.insert(solve.to_vec(state), SolutionSuffix(solution.clone(), idx));

        prev = state;
    }
}
