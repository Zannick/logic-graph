use crate::context::Ctx;
use crate::matchertrie::{MatcherDispatch, Node, Observable};
use crate::solutions::Solution;
use crate::world::{Exit, Location, World};
use std::fmt::Debug;
use std::sync::Arc;

pub trait Observation: Debug {
    type Ctx: Ctx;
    type Matcher: MatcherDispatch<
            Node = Node<Self::Matcher>,
            Struct = Self::Ctx,
            Value = (Arc<Solution<Self::Ctx>>, usize),
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
