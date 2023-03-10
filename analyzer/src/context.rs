use crate::world::*;
use sort_by_derive::SortBy;
use std::fmt::{self, format, Debug, Display};
use std::hash::Hash;
use std::rc::Rc;

pub trait Ctx: Clone + Eq + Debug {
    type World: World;
    type ItemId: Id;
    type AreaId: Id;
    type RegionId: Id;
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

    fn local_travel_time(&self, dest: <<Self::World as World>::Exit as Exit>::SpotId) -> i32;

    fn count_visits(&self) -> i32;
    fn count_skips(&self) -> i32;
    fn progress(&self) -> i32;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum History<T>
where
    T: Ctx,
{
    Warp(
        <<<T as Ctx>::World as World>::Warp as Warp>::WarpId,
        <<<T as Ctx>::World as World>::Exit as Exit>::SpotId,
    ),
    Get(
        <T as Ctx>::ItemId,
        <<<T as Ctx>::World as World>::Location as Location>::LocId,
    ),
    Move(<<<T as Ctx>::World as World>::Exit as Exit>::ExitId),
    MoveGet(
        <T as Ctx>::ItemId,
        <<<T as Ctx>::World as World>::Exit as Exit>::ExitId,
    ),
    MoveLocal(<<<T as Ctx>::World as World>::Exit as Exit>::SpotId),
    Activate(<<<T as Ctx>::World as World>::Action as Action>::ActionId),
}
impl<T> Copy for History<T> where T: Ctx {}

impl<T> Display for History<T>
where
    T: Ctx,
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
            History::Activate(action) => write!(f, "= Do {}", action),
        }
    }
}
impl<T> Hash for History<T>
where
    T: Ctx,
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct HistoryNode<T>
where
    T: Ctx,
{
    entry: History<T>,
    prev: Option<Rc<HistoryNode<T>>>,
}

struct HistoryIterator<T>
where
    T: Ctx,
{
    next: Option<Rc<HistoryNode<T>>>,
}
impl<T> Iterator for HistoryIterator<T>
where
    T: Ctx,
{
    type Item = History<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(hist) = self.next.clone() {
            self.next = hist.prev.clone();
            Some(hist.entry)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Explore,
    Check,
    Activate,
}

#[derive(Clone, Debug, SortBy)]
pub struct ContextWrapper<T>
where
    T: Ctx,
{
    ctx: T,
    #[sort_by]
    elapsed: i32,
    penalty: i32,
    history: Option<Rc<HistoryNode<T>>>,
    pub minimize: bool,
}

impl<T: Ctx> ContextWrapper<T> {
    pub fn new(ctx: T) -> ContextWrapper<T> {
        ContextWrapper {
            ctx,
            elapsed: 0,
            penalty: 0,
            history: None,
            minimize: false,
        }
    }

    pub fn append_history(&mut self, step: History<T>) {
        self.history = Some(Rc::new(HistoryNode {
            entry: step,
            prev: self.history.clone(),
        }))
    }

    pub fn history_rev(&self) -> impl Iterator<Item = History<T>> {
        HistoryIterator {
            next: self.history.clone(),
        }
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

    pub fn score(&self) -> i32 {
        // We want to sort by elapsed time, low to high: (X - elapsed)
        // with a bonus based on progress to prioritize states closer to the end:
        //   + progress * progress [progress in range 0..100]
        // penalty is added to states that
        self.ctx.progress() * self.ctx.progress() - self.elapsed - self.penalty
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
            if self.ctx.todo(canon_loc_id) {
                self.ctx.skip(canon_loc_id);
            }
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
        for canon_loc_id in world.get_canon_locations(loc.id()) {
            self.ctx.skip(canon_loc_id);
        }
        self.ctx.visit(loc.id());
        self.ctx.spend(loc.price());
        self.ctx.collect(loc.item());
        self.elapse(loc.time());
        self.ctx.spend(exit.price());
        self.ctx.set_position(exit.dest());
        self.elapse(exit.time());
        self.append_history(History::MoveGet(loc.item(), exit.id()));
    }

    pub fn activate<W, A>(&mut self, action: &A)
    where
        W: World<Action = A>,
        T: Ctx<World = W>,
        A: Action + Accessible<Context = T, Currency = <W::Location as Accessible>::Currency>,
    {
        action.perform(&mut self.ctx);
        self.elapse(action.time());
        self.ctx.spend(action.price());
        self.append_history(History::Activate(action.id()));
    }

    pub fn is_useful<W, A>(&self, action: &A) -> bool
    where
        W: World<Action = A>,
        T: Ctx<World = W>,
        A: Action<Context = T>,
    {
        if !action.has_effect(&self.ctx) {
            return false;
        }
        let mut prev = 1;
        if let Some(cycle) = action.cycle_length() {
            for last in self.history_rev() {
                match last {
                    History::Activate(a) => {
                        if a == action.id() {
                            prev += 1;
                            if prev >= cycle {
                                return false;
                            }
                        } else {
                            break;
                        }
                    }
                    History::Get(_, _) | History::MoveGet(_, _) => break,
                    _ => (),
                }
            }
        }
        true
    }

    pub fn info(&self) -> String {
        format(format_args!(
            "At {} after {}ms (score={}), visited={}, skipped={}, penalty={} last={}",
            self.ctx.position(),
            self.elapsed,
            self.score(),
            self.get().count_visits(),
            self.get().count_skips(),
            self.penalty,
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
}
