use crate::world::*;
use serde::{Deserialize, Serialize};
use std::fmt::{self, format, Debug, Display};
use std::hash::Hash;
use std::sync::Arc;

pub trait Ctx:
    Clone + Eq + Debug + Hash + Send + Sync + Serialize + for<'a> Deserialize<'a>
{
    type World: World;
    type ItemId: Id;
    type AreaId: Id;
    type RegionId: Id;
    type MovementState: Copy + Clone + Eq + Debug + Hash;
    const NUM_ITEMS: i32;

    fn has(&self, item: Self::ItemId) -> bool;
    fn count(&self, item: Self::ItemId) -> i16;
    fn collect(&mut self, item: Self::ItemId);

    fn position(&self) -> <<Self::World as World>::Exit as Exit>::SpotId;
    fn set_position(&mut self, pos: <<Self::World as World>::Exit as Exit>::SpotId);
    fn reload_game(&mut self);
    fn reset_all(&mut self);
    fn reset_region(&mut self, region_id: Self::RegionId);
    fn reset_area(&mut self, area_id: Self::AreaId);

    fn can_afford(&self, cost: &<<Self::World as World>::Location as Accessible>::Currency)
        -> bool;
    fn spend(&mut self, cost: &<<Self::World as World>::Location as Accessible>::Currency);

    fn visit(&mut self, loc_id: <<Self::World as World>::Location as Location>::LocId);
    fn skip(&mut self, loc_id: <<Self::World as World>::Location as Location>::LocId);
    fn reset(&mut self, loc_id: <<Self::World as World>::Location as Location>::LocId);
    fn todo(&self, loc_id: <<Self::World as World>::Location as Location>::LocId) -> bool;
    fn visited(&self, loc_id: <<Self::World as World>::Location as Location>::LocId) -> bool;
    fn skipped(&self, loc_id: <<Self::World as World>::Location as Location>::LocId) -> bool;

    fn all_spot_checks(&self, id: <<Self::World as World>::Exit as Exit>::SpotId) -> bool;
    fn all_area_checks(&self, id: Self::AreaId) -> bool;
    fn all_region_checks(&self, id: Self::RegionId) -> bool;

    fn get_movement_state(&self) -> Self::MovementState;
    fn local_travel_time(
        &self,
        movement_state: Self::MovementState,
        dest: <<Self::World as World>::Exit as Exit>::SpotId,
    ) -> i32;

    fn count_visits(&self) -> i32;
    fn count_skips(&self) -> i32;
    fn progress(&self) -> i32;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum History<ItemId, SpotId, LocId, ExitId, ActionId, WarpId> {
    Warp(WarpId, SpotId),
    Get(ItemId, LocId),
    Move(ExitId),
    MoveGet(ItemId, ExitId),
    MoveLocal(SpotId),
    Activate(ActionId),
}

impl<I, S, L, E, A, Wp> Copy for History<I, S, L, E, A, Wp>
where
    I: Id,
    S: Id,
    L: Id,
    E: Id,
    A: Id,
    Wp: Id,
{
}

impl<I, S, L, E, A, Wp> Display for History<I, S, L, E, A, Wp>
where
    I: Id,
    S: Id,
    L: Id,
    E: Id,
    A: Id,
    Wp: Id,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            History::Warp(warp, dest) => write!(f, "  {}warp to {}", warp, dest),
            History::Get(item, loc) => write!(f, "* Collect {} from {}", item, loc),
            History::Move(exit) => write!(f, "  Take exit {}", exit),
            History::MoveGet(item, exit) => {
                write!(f, "* Take hybrid exit {}, collecting {}", exit, item)
            }
            History::MoveLocal(spot) => write!(f, "  Move to {}", spot),
            History::Activate(action) => write!(f, "! Do {}", action),
        }
    }
}
impl<I, S, L, E, A, Wp> Hash for History<I, S, L, E, A, Wp>
where
    I: Hash,
    S: Hash,
    L: Hash,
    E: Hash,
    A: Hash,
    Wp: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            History::Warp(warp, dest) => {
                warp.hash(state);
                dest.hash(state);
            }
            History::Get(item, loc) => {
                item.hash(state);
                loc.hash(state);
            }
            History::Move(exit) => {
                exit.hash(state);
            }
            History::MoveGet(item, exit) => {
                item.hash(state);
                exit.hash(state);
            }
            History::MoveLocal(spot) => {
                spot.hash(state);
            }
            History::Activate(action) => {
                action.hash(state);
            }
        }
    }
}

pub type HistoryAlias<T> = History<
    <T as Ctx>::ItemId,
    <<<T as Ctx>::World as World>::Exit as Exit>::SpotId,
    <<<T as Ctx>::World as World>::Location as Location>::LocId,
    <<<T as Ctx>::World as World>::Exit as Exit>::ExitId,
    <<<T as Ctx>::World as World>::Action as Action>::ActionId,
    <<<T as Ctx>::World as World>::Warp as Warp>::WarpId,
>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct HistoryNode<I, S, L, E, A, Wp> {
    entry: History<I, S, L, E, A, Wp>,
    #[allow(clippy::type_complexity)]
    prev: Option<Arc<HistoryNode<I, S, L, E, A, Wp>>>,
}

type HistoryNodeAlias<T> = HistoryNode<
    <T as Ctx>::ItemId,
    <<<T as Ctx>::World as World>::Exit as Exit>::SpotId,
    <<<T as Ctx>::World as World>::Location as Location>::LocId,
    <<<T as Ctx>::World as World>::Exit as Exit>::ExitId,
    <<<T as Ctx>::World as World>::Action as Action>::ActionId,
    <<<T as Ctx>::World as World>::Warp as Warp>::WarpId,
>;

struct HistoryIterator<T>
where
    T: Ctx,
{
    next: Option<Arc<HistoryNodeAlias<T>>>,
}
impl<T> Iterator for HistoryIterator<T>
where
    T: Ctx,
{
    type Item = HistoryAlias<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(hist) = self.next.clone() {
            self.next = hist.prev.clone();
            Some(hist.entry)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseContextWrapper<T, I, S, L, E, A, Wp> {
    ctx: T,
    elapsed: i32,
    penalty: i32,
    bonus: i32,
    // Rc is not Sync; if this poses a problem for HeapDB we'll have to change it to Arc
    // or make a type for ContextWrapper to convert into
    #[allow(clippy::type_complexity)]
    history: Option<Arc<HistoryNode<I, S, L, E, A, Wp>>>,
}

pub type ContextWrapper<T> = BaseContextWrapper<
    T,
    <T as Ctx>::ItemId,
    <<<T as Ctx>::World as World>::Exit as Exit>::SpotId,
    <<<T as Ctx>::World as World>::Location as Location>::LocId,
    <<<T as Ctx>::World as World>::Exit as Exit>::ExitId,
    <<<T as Ctx>::World as World>::Action as Action>::ActionId,
    <<<T as Ctx>::World as World>::Warp as Warp>::WarpId,
>;

impl<T: Ctx> ContextWrapper<T> {
    pub fn new(ctx: T) -> ContextWrapper<T> {
        ContextWrapper {
            ctx,
            elapsed: 0,
            penalty: 0,
            bonus: 0,
            history: None,
        }
    }

    pub fn append_history(&mut self, step: HistoryAlias<T>) {
        self.history = Some(Arc::new(HistoryNode {
            entry: step,
            prev: self.history.clone(),
        }))
    }

    pub fn history_rev(&self) -> impl Iterator<Item = HistoryAlias<T>> {
        HistoryIterator::<T> {
            next: self.history.clone(),
        }
    }

    pub fn last_step(&self) -> Option<HistoryAlias<T>> {
        self.history.as_ref().map(|node| node.entry)
    }

    pub fn elapse(&mut self, t: i32) {
        self.elapsed += t;
    }

    pub fn elapsed(&self) -> i32 {
        self.elapsed
    }

    pub fn penalize(&mut self, penalty: i32) {
        self.penalty += penalty;
    }

    pub fn penalty(&self) -> i32 {
        self.penalty
    }

    pub fn reward(&mut self, bonus: i32) {
        self.bonus += bonus;
    }

    pub fn bonus(&self) -> i32 {
        self.bonus
    }

    pub fn score(&self, scale_factor: i32) -> i32 {
        // We want to sort by elapsed time, low to high:
        // with a bonus based on progress to prioritize states closer to the end:
        //   + 50 * progress * progress [progress in range 0..100]
        //   i.e. 0 to 500,000
        // (on the order of the real max time seems good)
        // However, we want to make sure we get variety in the early-to-mid game,
        // so we have to prioritize low-progress/low-elapsed times.
        // penalty is added to states that do really inefficient things
        // and to deprioritize actions
        let progress = self.ctx.progress();
        -self.elapsed - self.penalty + self.bonus
            + if progress > 25 {
                scale_factor * progress * progress
            } else {
                scale_factor * 25 * 25
            }
    }

    pub fn get(&self) -> &T {
        &self.ctx
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.ctx
    }

    pub fn visit<W, L>(&mut self, world: &W, loc: &L)
    where
        W: World<Location = L>,
        T: Ctx<World = W>,
        L: Location<Context = T>,
    {
        self.ctx.visit(loc.id());
        self.ctx.collect(loc.item());
        self.ctx.spend(loc.price());
        for canon_loc_id in world.get_canon_locations(loc.id()) {
            self.ctx.skip(canon_loc_id);
        }
        self.elapse(loc.time());
        self.append_history(History::Get(loc.item(), loc.id()));
    }

    pub fn exit<W, E>(&mut self, exit: &E)
    where
        W: World<Exit = E>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    {
        self.ctx.set_position(exit.dest());
        self.elapse(exit.time());
        self.ctx.spend(exit.price());
        self.append_history(History::Move(exit.id()));
    }

    pub fn move_local<W, E>(&mut self, spot: E::SpotId, time: i32)
    where
        W: World<Exit = E>,
        T: Ctx<World = W>,
        E: Exit<Context = T>,
    {
        self.ctx.set_position(spot);
        self.elapse(time);
        self.append_history(History::MoveLocal(spot))
    }

    pub fn warp<W, E, Wp>(&mut self, warp: &Wp)
    where
        W: World<Exit = E, Warp = Wp>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
        Wp: Warp<
            SpotId = <E as Exit>::SpotId,
            Context = T,
            Currency = <W::Location as Accessible>::Currency,
        >,
    {
        warp.prewarp(&mut self.ctx);
        self.ctx.set_position(warp.dest(&self.ctx));
        self.elapse(warp.time());
        self.ctx.spend(warp.price());
        if warp.should_reload() {
            self.ctx.reload_game();
        }
        self.append_history(History::Warp(warp.id(), warp.dest(&self.ctx)));
    }

    pub fn visit_exit<W, L, E>(&mut self, world: &W, loc: &L, exit: &E)
    where
        W: World<Exit = E, Location = L>,
        T: Ctx<World = W>,
        L: Location<Context = T>,
        E: Exit<Context = T, Currency = L::Currency>,
    {
        self.ctx.visit(loc.id());
        self.ctx.spend(loc.price());
        self.ctx.collect(loc.item());
        self.elapse(loc.time());
        self.ctx.spend(exit.price());
        self.ctx.set_position(exit.dest());
        self.elapse(exit.time());

        for canon_loc_id in world.get_canon_locations(loc.id()) {
            self.ctx.skip(canon_loc_id);
        }
        self.append_history(History::MoveGet(loc.item(), exit.id()));
    }

    pub fn activate<W, A>(&mut self, action: &A)
    where
        W: World<Action = A>,
        T: Ctx<World = W>,
        A: Action<Context = T, Currency = <W::Location as Accessible>::Currency>,
    {
        action.perform(&mut self.ctx);
        self.elapse(action.time());
        self.ctx.spend(action.price());
        self.append_history(History::Activate(action.id()));
    }

    pub fn replay<W, L, E, Wp>(&mut self, world: &W, step: HistoryAlias<T>)
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency, LocId = L::LocId>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        // We skip checking validity ahead of time, i.e. can_access.
        // Some other times we should still assert some possibility.
        match step {
            History::Warp(wp, dest) => {
                self.warp(world.get_warp(wp));
                assert!(
                    self.get().position() == dest,
                    "Invalid replay: warp {:?}",
                    wp
                );
            }
            History::Get(item, loc_id) => {
                let loc = world.get_location(loc_id);
                self.visit(world, loc);
                assert!(loc.item() == item, "Invalid replay: visit {:?}", loc_id);
            }
            History::Move(exit_id) => {
                let exit = world.get_exit(exit_id);
                self.exit(exit);
            }
            History::MoveGet(item, exit_id) => {
                let exit = world.get_exit(exit_id);
                let loc =
                    world.get_location(exit.loc_id().expect("MoveGet requires a hybrid exit"));
                self.visit_exit(world, loc, exit);
                assert!(
                    loc.item() == item,
                    "Invalid replay: visit-exit {:?}",
                    exit_id
                )
            }
            History::MoveLocal(spot) => {
                let movement_state = self.ctx.get_movement_state();
                let time = self.ctx.local_travel_time(movement_state, spot);
                assert!(time >= 0, "Invalid replay: move-local {:?}", spot);
                self.move_local(spot, time);
            }
            History::Activate(act_id) => {
                let action = world.get_action(act_id);
                self.activate(action);
            }
        }
    }

    pub fn info(&self, scale_factor: i32) -> String {
        format(format_args!(
            "At {}ms (score={}), visited={}, skipped={}, penalty={}, bonus={}\nNow: {} after {}",
            self.elapsed,
            self.score(scale_factor),
            self.get().count_visits(),
            self.get().count_skips(),
            self.penalty,
            self.bonus,
            self.ctx.position(),
            if let Some(val) = &self.history {
                val.entry.to_string()
            } else {
                String::from("None")
            },
        ))
    }

    pub fn history_str(&self) -> String {
        let mut vec: Vec<String> = self
            .history_rev()
            .map(|h| h.to_string())
            .collect::<Vec<String>>();
        vec.reverse();
        vec.join("\n")
    }

    pub fn history_preview(&self) -> String {
        let mut vec: Vec<String> = self
            .history_rev()
            .filter_map(|h| match h {
                History::Get(..) | History::MoveGet(..) => Some(h.to_string()),
                _ => None,
            })
            .collect::<Vec<String>>();
        vec.reverse();
        vec.join("\n")
    }

    pub fn history_summary(&self) -> String {
        let mut vec: Vec<String> = self
            .history_rev()
            .fold(Vec::new(), |mut v, h| {
                if let Some(lh) = v.last() {
                    match (*lh, h) {
                        (
                            History::Move(..) | History::MoveLocal(..),
                            History::Move(..) | History::MoveLocal(..),
                        ) => (),
                        _ => v.push(h),
                    }
                } else {
                    v.push(h);
                };
                v
            })
            .into_iter()
            .map(|h| match h {
                History::Get(..) | History::MoveGet(..) | History::Activate(..) => h.to_string(),
                History::Move(e) => format!("  Move... to {}", e),
                History::MoveLocal(s) => {
                    format!("  Move... to {}", s)
                }
                History::Warp(w, s) => {
                    format!("  {}warp to {}", w, s)
                }
            })
            .collect();
        vec.reverse();
        vec.join("\n")
    }
}
