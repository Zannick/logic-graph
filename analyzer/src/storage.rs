use crate::context::*;
use crate::estimates::ContextScorer;
use crate::scoring::*;
use crate::steiner::*;
use crate::world::*;
use anyhow::Result;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};


/// The key for a T (Ctx) in the statedb, and the value in the queue db
/// are all T itself.
pub(crate) fn serialize_state<T: Ctx>(el: &T) -> Vec<u8> {
    let mut key = Vec::with_capacity(std::mem::size_of::<T>());
    el.serialize(&mut Serializer::new(&mut key)).unwrap();
    key
}

pub(crate) fn deserialize_state<T: Ctx>(buf: &[u8]) -> Result<T> {
    Ok(rmp_serde::from_slice::<T>(buf)?)
}

pub fn serialize_data<V>(v: V) -> Vec<u8>
where
    V: Serialize,
{
    let mut val = Vec::with_capacity(std::mem::size_of::<V>());
    v.serialize(&mut Serializer::new(&mut val)).unwrap();
    val
}

pub(crate) fn get_obj_from_data<V>(buf: &[u8]) -> Result<V>
where
    V: for<'de> Deserialize<'de>,
{
    Ok(rmp_serde::from_slice::<V>(buf)?)
}

/// Common functionality all DBs must support.
///
/// Conceptually, a state can be:
///     - *queued* if it is in the in-memory heap
///     - *processed* if its child states have been created (only needs to happen once per state ever)
///     - *preserved* if it is *unqueued* and *unprocessed*; not in the in-memory queue but should still be retrievable
///     - *available* if it is preserved and under the max time limit (states over the limit won't be counted in some functions)
pub trait ContextDB<'w, W, T, const KS: usize, SM>: Sync
where
    W: World + 'w,
    T: Ctx<World = W>,
    SM: ScoreMetric<'w, W, T, KS> + 'w,
{
    const NAME: &'static str;
    fn name(&self) -> &'static str {
        Self::NAME
    }

    // region: Scoring

    /// Returns a reference to the metric used for scoring.
    fn metric(&self) -> &SM;

    /// Returns a reference to the context scorer used by the DB.
    fn scorer(
        &self,
    ) -> &ContextScorer<
        'w,
        W,
        <<W as World>::Exit as Exit>::SpotId,
        <<W as World>::Location as Location>::LocId,
        <<W as World>::Location as Location>::CanonId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    > {
        self.metric().estimator()
    }
    // endregion

    // region: Stats

    /// Returns the number of preserved elements in the queue.
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of states in the db.
    fn seen(&self) -> usize;

    /// Returns the number of processed states in the db.
    fn processed(&self) -> usize;

    /// Returns the score of the best preserved element at the given progress level in the db.
    ///
    /// A score of `u32::MAX` means there are no preserved elements at that progress level.
    fn preserved_best(&self, progress: usize) -> u32;

    /// Returns the scores of the best preserved element at each progress level in the db.
    ///
    /// A score of `u32::MAX` means there are no preserved elements at that progress level.
    fn preserved_bests(&self) -> Vec<u32>;

    /// Returns the first progress level at which there are preserved elements in the db.
    fn min_preserved_progress(&self) -> Option<usize>;

    /// Print data graphs.
    fn print_graphs(&self) -> Result<()>;

    /// Get extra stats details about actions performed or not performed.
    fn extra_stats(&self) -> String;
    // endregion

    // region: Time

    /// Max allowable time for a route.
    ///
    /// States with scores that exceed this should not be processed.
    fn max_time(&self) -> u32;
    /// Sets the max time to the given limit.
    fn set_max_time(&self, max_time: u32);
    /// Sets the max time to the given limit plus a small additional buffer.
    fn set_lenient_max_time(&self, max_time: u32) {
        self.set_max_time(max_time + (max_time / 1024))
    }
    // endregion

    // region: Reads

    /// Returns the best times recorded to reach the given encoded state.
    /// May only be called with an encoded state known to exist in the db.
    fn get_best_times_raw(&self, state_key: &[u8]) -> Result<BestTimes>;
    /// Returns the best times recorded to reach the given state.
    /// May only be called with a state known to exist in the db.
    fn get_best_times(&self, el: &T) -> Result<BestTimes> {
        self.get_best_times_raw(&serialize_state(el))
    }

    /// Lookup the state's estimated remaining time in the db, or consult
    /// the metric's estimator.
    fn estimated_remaining_time(&self, ctx: &T) -> u32;

    /// Returns the best score recorded for the given encoded state.
    fn lookup_score_raw(&self, key: &[u8]) -> Result<SM::Score> {
        Ok(SM::score_from_times(self.get_best_times_raw(key)?))
    }
    /// Returns the best score recorded for the given state.
    fn lookup_score(&self, el: &T) -> Result<SM::Score> {
        Ok(SM::score_from_times(self.get_best_times(el)?))
    }

    /// Returns whether the given encoded state has been processed.
    fn was_processed_raw(&self, key: &[u8]) -> Result<bool>;
    /// Returns whether the given state has been processed.
    fn was_processed(&self, el: &T) -> Result<bool> {
        self.was_processed_raw(&serialize_state(el))
    }

    /// Returns the best route in the db to reach the given encoded state, and the route's total elapsed time.
    fn get_history_raw(&self, state_key: &Vec<u8>) -> Result<(Vec<HistoryAlias<T>>, u32)>;
    /// Returns the best route in the db to reach the given state, and the route's total elapsed time.
    fn get_history(&self, el: &T) -> Result<(Vec<HistoryAlias<T>>, u32)> {
        self.get_history_raw(&serialize_state(el))
    }
    /// Returns the last step in the history of a state.
    fn get_last_history_step(&self, el: &T) -> Result<Option<HistoryAlias<T>>>;
    /// Returns the last step in the history of a wrapped state (which might be in the wrapper).
    fn get_last_history_step_wrapper(
        &self,
        ctx: &ContextWrapper<T>,
    ) -> Result<Option<HistoryAlias<T>>> {
        if let Some(h) = ctx.recent_history().last() {
            Ok(Some(*h))
        } else {
            self.get_last_history_step(ctx.get())
        }
    }
    // endregion

    // Writes

    /// Records an element in the db and (if new) marks it as preserved (unqueued and unprocessed).
    /// 
    /// See `record_one` for adding an element and marking it as queued instead.
    fn push(&self, el: ContextWrapper<T>, prev: Option<&T>) -> Result<()>;

    /// Retrieves a single preserved element, marks it as queued, and returns it wrapped for processing.
    /// It will be the one with the lowest score at the lowest progress level that's >= `start_progress`.
    ///
    /// Returns `Ok(None)` if there are no preserved elements in the db.
    fn pop(&self, start_progress: usize) -> Result<Option<ContextWrapper<T>>>;

    /// Marks the given elements as not queued and ensures they are preserved
    /// for eventual retrieval. Requires that these elements have been recorded previously.
    fn evict(&self, iter: impl IntoIterator<Item = (T, SM::Score)>) -> Result<()>;

    /// Retrieves up to `count` elements from the database, starting from the given progress level,
    /// marking them as queued. Elements returned will not exceed the given total estimated time limit.
    fn retrieve(
        &self,
        start_progress: usize,
        count: usize,
        time_limit: u32,
    ) -> Result<Vec<(T, SM::Score)>>;

    /// Records the state, potentially updating its score if the state has been seen previously,
    /// and marking it as queued if it is new.
    ///
    /// Returns the score of the state if this was the best score seen so far,
    /// or None otherwise (suggesting the state does not need to be requeued).
    ///
    /// The state object will be modified to clear the saved history.
    /// 
    /// See `push` for recording an element and marking it unqueued instead.
    fn record_one(&self, el: &mut ContextWrapper<T>, prev: Option<&T>)
        -> Result<Option<SM::Score>>;

    /// Records the processing of a state and the generated children of that state.
    ///
    /// Returns for each state its score, if it was the best score seen so far,
    /// or None otherwise (suggesting the state does not need to be requeued).
    ///
    /// The states will be modified to clear their saved histories, and the list of states
    /// may be sorted.
    fn record_processed(
        &self,
        prev: &T,
        states: &mut Vec<ContextWrapper<T>>,
    ) -> Result<Vec<Option<SM::Score>>>;

    // Recovery

    /// Whether the db is in recovery.
    /// This should return false after `restore()` completes, but can be read from a separate thread.
    fn recovery(&self) -> bool;

    /// Perform any necessary actions to recover the database from an unexpected showdown
    /// (such as marking queued elements as unqueued, as we've likely lost the queue; or
    /// recalculating db-wide analytics; etc).
    fn restore(&self);
}
