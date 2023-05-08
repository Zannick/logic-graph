use crate::world::*;
use crate::{new_hashmap, CommonHasher};
use as_slice::{AsSlice, AsMutSlice};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, format, Debug, Display};
use std::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;

pub trait Ctx:
    Clone + Eq + Debug + Hash + Send + Sync + Serialize + for<'a> Deserialize<'a>
{
    type World: World;
    type ItemId: Id + Default;
    type AreaId: Id;
    type RegionId: Id;
    type MovementState: Copy + Clone + Eq + Debug + Hash + AsSlice<Element = bool> + AsMutSlice<Element = bool>;
    const NUM_ITEMS: u32;

    fn is_subset(sub: Self::MovementState, sup: Self::MovementState) -> bool {
        let s1 = AsSlice::as_slice(&sub);
        let s2 = AsSlice::as_slice(&sup);
        // sup <= sup if for all (a,b), a is false or b is true
        s1.len() == s2.len() && s1.iter().zip(s2.iter()).all(|(a, b)| *b || !*a )
    }

    fn combine(mut ms1: Self::MovementState, ms2: Self::MovementState) -> Self::MovementState {
        for (m, m2) in ms1.as_mut_slice().iter_mut().zip(ms2.as_slice().iter()) {
            *m = *m || *m2;
        }
        ms1
    }

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
    ) -> u32;

    fn count_visits(&self) -> u32;
    fn count_skips(&self) -> u32;
    fn progress(&self) -> u32;
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

fn extract_match<'c, 's>(c: &'c Captures<'s>, g: &'s str, s: &'s str) -> Result<&'s str, String> {
    if let Some(m) = c.name(g) {
        Ok(m.as_str())
    } else {
        Err(format!("Group '{}' not matched: {}", g, s))
    }
}

impl<I, S, L, E, A, Wp> FromStr for History<I, S, L, E, A, Wp>
where
    I: FromStr<Err = String> + Default,
    S: FromStr<Err = String>,
    L: FromStr<Err = String>,
    E: FromStr<Err = String>,
    A: FromStr<Err = String>,
    Wp: FromStr<Err = String>,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref WARP: Regex = Regex::new(
                // Warp
                //   EarthSavewarp to Antarctica > West > Helipad
                r"(?P<warp>[^:]+)[Ww]arp to (?P<spot>.*$)").unwrap();
            static ref GET: Regex = Regex::new(
                // Get
                // * Collect Station_Power from Antarctica > Power Room > Switch: Flip
                r"(?:\* )?(?:[Cc]ollect (?P<item>\w+) from|[Vv]isit) (?P<loc>.*$)").unwrap();
            static ref MOVE: Regex = Regex::new(
                // Move
                //   Move... to Antarctica > West > Shed Entry ==> Shed > Interior (1)
                r"[Mm]ove(?:\.\.\.)? to (?P<exit>.* ==> .*$)").unwrap();
            static ref MOVE_GET: Regex = Regex::new(
                // MoveGet
                // * Take hybrid exit Glacier > The Big Drop > Water Surface: Drown, collecting Amashilama
                r"(?:\* )?Take hybrid exit (?P<exit>.*?)(?:, collecting (?P<item>.*$))").unwrap();
            static ref MOVE_LOCAL: Regex = Regex::new(
                // MoveLocal
                //   Move... to Antarctica > Power Room > Switch
                r"[Mm]ove(?:\.\.\.)? to (?P<spot>.*$)").unwrap();
            static ref ACTIVATE: Regex = Regex::new(
                // Activate
                // ! Do Amagi > Main Area > Carving: Key Combo
                r"(?:! )? (?:[Dd]o|[Aa]ctivate) (?P<action>.*$)").unwrap();
        }
        if let Some(cap) = WARP.captures(s) {
            let warp = extract_match(&cap, "warp", s)?;
            let spot = extract_match(&cap, "spot", s)?;
            Ok(History::Warp(
                <Wp as FromStr>::from_str(warp)?,
                <S as FromStr>::from_str(spot)?,
            ))
        } else if let Some(cap) = GET.captures(s) {
            let item = extract_match(&cap, "item", s).unwrap_or_default();
            let loc = extract_match(&cap, "loc", s)?;
            Ok(History::Get(
                <I as FromStr>::from_str(item).unwrap_or_default(),
                <L as FromStr>::from_str(loc)?,
            ))
        } else if let Some(cap) = MOVE.captures(s) {
            let exit = extract_match(&cap, "exit", s)?;
            Ok(History::Move(<E as FromStr>::from_str(exit)?))
        } else if let Some(cap) = MOVE_GET.captures(s) {
            let exit = extract_match(&cap, "exit", s)?;
            let item = extract_match(&cap, "item", s).unwrap_or_default();
            Ok(History::MoveGet(
                <I as FromStr>::from_str(item).unwrap_or_default(),
                <E as FromStr>::from_str(exit)?,
            ))
        } else if let Some(cap) = MOVE_LOCAL.captures(s) {
            let spot = extract_match(&cap, "spot", s)?;
            Ok(History::MoveLocal(<S as FromStr>::from_str(spot)?))
        } else if let Some(cap) = ACTIVATE.captures(s) {
            let action = extract_match(&cap, "action", s)?;
            Ok(History::Activate(<A as FromStr>::from_str(action)?))
        } else {
            Err(format!("History<T> did not find a match for: {}", s))
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

pub trait Wrapper<T> {
    fn get(&self) -> &T;
    fn elapsed(&self) -> u32;
}
pub struct HistoryArchive<I, S, L, E, A, Wp> {
    next: usize,
    archive: HashMap<usize, Arc<HistoryNode<I, S, L, E, A, Wp>>, CommonHasher>,
}

#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivedContextWrapper<T> {
    ctx: T,
    elapsed: u32,
    hist_archive: usize,
}
impl<T> Wrapper<T> for ArchivedContextWrapper<T> {
    fn get(&self) -> &T {
        &self.ctx
    }
    fn elapsed(&self) -> u32 {
        self.elapsed
    }
}

impl<I, S, L, E, A, Wp> HistoryArchive<I, S, L, E, A, Wp>
where
    I: Hash,
    S: Hash,
    L: Hash,
    E: Hash,
    A: Hash,
    Wp: Hash,
{
    pub fn new() -> Self {
        Self {
            next: 1, // reserve 0 for the None
            archive: new_hashmap(),
        }
    }

    pub fn len(&self) -> usize {
        self.archive.len()
    }

    pub fn counted(&self) -> usize {
        self.next
    }

    pub fn archive<T>(
        &mut self,
        BaseContextWrapper {
            ctx,
            elapsed,
            history,
        }: BaseContextWrapper<T, I, S, L, E, A, Wp>,
    ) -> ArchivedContextWrapper<T> {
        if let Some(hist) = history {
            let hist_archive = self.next;
            self.next += 1;
            self.archive.insert(hist_archive, hist);
            ArchivedContextWrapper {
                ctx,
                elapsed,
                hist_archive,
            }
        } else {
            ArchivedContextWrapper {
                ctx,
                elapsed,
                hist_archive: 0,
            }
        }
    }

    pub fn retrieve<T>(
        &mut self,
        ArchivedContextWrapper {
            ctx,
            elapsed,
            hist_archive,
        }: ArchivedContextWrapper<T>,
    ) -> BaseContextWrapper<T, I, S, L, E, A, Wp> {
        let history = self.archive.remove(&hist_archive);
        if hist_archive > 0 {
            assert!(
                history.is_some(),
                "Attempted to retrieve missing history entry {}",
                hist_archive
            );
        }
        BaseContextWrapper {
            ctx,
            elapsed,
            history,
        }
    }

    pub fn remove<T>(
        &mut self,
        ArchivedContextWrapper {
            ctx: _,
            elapsed: _,
            hist_archive,
        }: ArchivedContextWrapper<T>,
    ) {
        self.archive.remove(&hist_archive);
    }
}
pub type HistoryArchiveAlias<T, W> = HistoryArchive<
    <T as Ctx>::ItemId,
    <<W as World>::Exit as Exit>::SpotId,
    <<W as World>::Location as Location>::LocId,
    <<W as World>::Exit as Exit>::ExitId,
    <<W as World>::Action as Action>::ActionId,
    <<W as World>::Warp as Warp>::WarpId,
>;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BaseContextWrapper<T, I, S, L, E, A, Wp> {
    ctx: T,
    elapsed: u32,
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

impl<T: Ctx> Wrapper<T> for ContextWrapper<T> {
    fn get(&self) -> &T {
        &self.ctx
    }
    fn elapsed(&self) -> u32 {
        self.elapsed
    }
}

impl<T: Ctx> ContextWrapper<T> {
    pub fn new(ctx: T) -> ContextWrapper<T> {
        ContextWrapper {
            ctx,
            elapsed: 0,
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

    pub fn elapse(&mut self, t: u32) {
        self.elapsed += t;
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

    pub fn move_local<W, E>(&mut self, spot: E::SpotId, time: u32)
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
                assert!(time != u32::MAX, "Invalid replay: move-local {:?}", spot);
                self.move_local(spot, time);
            }
            History::Activate(act_id) => {
                let action = world.get_action(act_id);
                self.activate(action);
            }
        }
    }

    pub fn info(&self, est: u32) -> String {
        format(format_args!(
            "At {}ms (elapsed={} est. left={}), visited={}, skipped={}\nNow: {} after {}",
            self.elapsed + est,
            self.elapsed,
            est,
            self.get().count_visits(),
            self.get().count_skips(),
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
