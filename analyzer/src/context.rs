use crate::condense::CondensedEdge;
use crate::observer::Observer;
use crate::solutions::Solution;
use crate::world::*;
use bitflags::Flags;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::fmt::{self, format, Debug, Display};
use std::hash::Hash;
use std::ops::RangeInclusive;
use std::str::FromStr;
use std::sync::Arc;
use yaml_rust::Yaml;

pub trait Ctx:
    Clone
    + Eq
    + Debug
    + Default
    + Hash
    + Send
    + Sync
    + Serialize
    + for<'a> Deserialize<'a>
    + crate::matchertrie::Observable
{
    type World: World;
    type ItemId: Id + Default;
    type AreaId: Id;
    type RegionId: Id;
    type MovementState: Copy + Clone + Eq + Debug + Hash + Flags;
    type Observer: Observer<Ctx = Self>;
    type Expectation: Copy + Clone + Debug + Eq + Send;
    const NUM_ITEMS: u32;

    fn is_subset(sub: Self::MovementState, sup: Self::MovementState) -> bool {
        sup.contains(sub)
    }

    fn combine(ms1: Self::MovementState, ms2: Self::MovementState) -> Self::MovementState {
        ms1.union(ms2)
    }

    fn has(&self, item: Self::ItemId) -> bool;
    fn count(&self, item: Self::ItemId) -> i16;
    fn collect(&mut self, item: Self::ItemId, world: &Self::World);
    // test helper for items
    fn add_item(&mut self, item: Self::ItemId);
    // test helper for context vars
    fn parse_set_context(&mut self, ckey: &str, cval: &Yaml) -> Result<(), String>;
    fn parse_expect_context(ckey: &str, cval: &Yaml) -> Result<Self::Expectation, String>;
    fn assert_expectations(&self, exps: &Vec<Self::Expectation>) -> Result<(), String>;

    //fn build_verify_func(ckey: &str, cval: &Yaml) -> Result<impl Fn(&Self) -> bool, String>;

    fn position(&self) -> <<Self::World as World>::Exit as Exit>::SpotId;
    fn set_position(
        &mut self,
        pos: <<Self::World as World>::Exit as Exit>::SpotId,
        world: &Self::World,
    );
    // for testing only, skips enter handlers
    fn set_position_raw(&mut self, pos: <<Self::World as World>::Exit as Exit>::SpotId);

    fn reload_game(&mut self, world: &Self::World);
    fn reset_all(&mut self, world: &Self::World);
    fn reset_region(&mut self, region_id: Self::RegionId, world: &Self::World);
    fn reset_area(&mut self, area_id: Self::AreaId, world: &Self::World);

    fn can_afford(&self, cost: &<<Self::World as World>::Location as Accessible>::Currency)
        -> bool;
    fn amount_could_afford(
        &self,
        cost: &<<Self::World as World>::Location as Accessible>::Currency,
    ) -> i16;
    fn spend(&mut self, cost: &<<Self::World as World>::Location as Accessible>::Currency);
    fn observe_afford(
        &self,
        cost: &<<Self::World as World>::Location as Accessible>::Currency,
        observer: &mut Self::Observer,
    );

    fn visit(&mut self, loc_id: <<Self::World as World>::Location as Location>::LocId);
    fn reset(&mut self, loc_id: <<Self::World as World>::Location as Location>::LocId);
    fn take_exit(&mut self, exit: &<Self::World as World>::Exit, world: &Self::World);
    fn todo(&self, loc: &<Self::World as World>::Location) -> bool {
        !loc.skippable() && !self.visited(loc.id())
    }
    fn visited(&self, loc_id: <<Self::World as World>::Location as Location>::LocId) -> bool;

    fn all_spot_checks(&self, id: <<Self::World as World>::Exit as Exit>::SpotId) -> bool;
    fn all_area_checks(&self, id: Self::AreaId) -> bool;
    fn all_region_checks(&self, id: Self::RegionId) -> bool;

    fn get_movement_state(&self, world: &Self::World) -> Self::MovementState;
    fn observe_movement_state(
        &self,
        world: &Self::World,
        observer: &mut Self::Observer,
    ) -> Self::MovementState;
    fn local_travel_time(
        &self,
        movement_state: Self::MovementState,
        dest: <<Self::World as World>::Exit as Exit>::SpotId,
    ) -> u32;

    fn count_visits(&self) -> usize;
    fn progress(&self) -> u32;

    fn diff(&self, old: &Self) -> String;

    /// Observes the access checks and, if they pass, any side effects of the step.
    fn observe_replay<L, E, Wp>(
        &self,
        world: &Self::World,
        step: HistoryAlias<Self>,
        observer: &mut Self::Observer,
    ) -> bool
    where
        Self::World: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = Self>,
        E: Exit<Context = Self, Currency = <L as Accessible>::Currency>,
        Wp: Warp<
            SpotId = <E as Exit>::SpotId,
            Context = Self,
            Currency = <L as Accessible>::Currency,
        >,
    {
        match step {
            History::W(wp, dest) => {
                let warp = world.get_warp(wp);
                if warp.dest(self, world) == dest && warp.observe_access(self, world, observer) {
                    warp.observe_effects(self, world, observer);
                    observer.observe_on_entry(self, dest, world);
                    true
                } else {
                    false
                }
            }
            History::G(item, loc_id) | History::V(item, loc_id, ..) => {
                let spot_id = world.get_location_spot(loc_id);
                let loc = world.get_location(loc_id);
                if spot_id == self.position()
                    && loc.item() == item
                    && loc.observe_access(self, world, observer)
                {
                    observer.observe_visit(loc_id);
                    observer.observe_collect(self, item, world);
                    true
                } else {
                    false
                }
            }
            History::E(exit_id) => {
                let spot_id = world.get_exit_spot(exit_id);
                let exit = world.get_exit(exit_id);
                if spot_id == self.position() && exit.observe_access(self, world, observer) {
                    observer.observe_on_entry(self, exit.dest(), world);
                    true
                } else {
                    false
                }
            }
            History::L(spot_id) => {
                let movement_state = self.observe_movement_state(world, observer);
                let (best_free, best_mvmts) = Self::World::best_movements(self.position(), spot_id);
                if self.position() != spot_id
                    && Self::World::same_area(self.position(), spot_id)
                    && (best_free.is_some()
                        || best_mvmts
                            .into_iter()
                            .any(|(m, _)| Self::is_subset(m, movement_state)))
                {
                    observer.observe_on_entry(self, spot_id, world);
                    true
                } else {
                    false
                }
            }
            History::A(act_id) => {
                let spot_id = world.get_action_spot(act_id);
                let action = world.get_action(act_id);
                if (world.is_global_action(act_id) || self.position() == spot_id)
                    && action.observe_access(self, world, observer)
                {
                    action.observe_effects(self, world, observer);
                    let dest = action.dest(self, world);
                    if dest != spot_id && dest != <E as Exit>::SpotId::default() {
                        observer.observe_on_entry(self, dest, world);
                    }
                    true
                } else {
                    false
                }
            }
            History::C(spot_id, idx) => {
                let movement_state = self.observe_movement_state(world, observer);
                let edges = world.get_condensed_edges_from(self.position());
                let edge = &edges[idx];
                if edge.dst == spot_id && edge.observe_access(world, self, movement_state, observer)
                {
                    observer.observe_on_entry(self, spot_id, world);
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum History<ItemId, SpotId, LocId, ExitId, ActionId, WarpId> {
    // Warp
    W(WarpId, SpotId),
    // Get
    G(ItemId, LocId),
    // Get and Move
    V(ItemId, LocId, SpotId),
    // Exit
    E(ExitId),
    // Local movement
    L(SpotId),
    // Action
    A(ActionId),
    // Condensed local movement
    C(SpotId, usize),
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
            History::W(warp, dest) => write!(f, "  {}warp to {}", warp, dest),
            History::G(item, loc) => write!(f, "* Collect {} from {}", item, loc),
            History::V(item, loc, dest) => {
                write!(f, "* Collect {} from {} ==> {}", item, loc, dest)
            }
            History::E(exit) => write!(f, "  Take exit {}", exit),
            History::L(spot) => write!(f, "  Move to {}", spot),
            History::A(action) => write!(f, "! Do {}", action),
            History::C(spot, ..) => write!(f, "  Move... to {}", spot),
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
            History::W(warp, dest) => {
                warp.hash(state);
                dest.hash(state);
            }
            History::G(item, loc) => {
                item.hash(state);
                loc.hash(state);
            }
            History::V(item, loc, dest) => {
                item.hash(state);
                loc.hash(state);
                dest.hash(state);
            }
            History::E(exit) => {
                exit.hash(state);
            }
            History::L(spot) => {
                spot.hash(state);
            }
            History::C(spot, idx) => {
                spot.hash(state);
                idx.hash(state);
            }
            History::A(action) => {
                action.hash(state);
            }
        }
    }
}

fn extract_match<'c, 's>(c: &'c Captures<'s>, g: &'s str, s: &'s str) -> Result<&'s str, String> {
    if let Some(m) = c.name(g) {
        Ok(m.as_str().trim_end())
    } else {
        Err(format!("Group '{}' not matched: {}", g, s))
    }
}

impl<I, S, L, E, A, Wp> FromStr for History<I, S, L, E, A, Wp>
where
    I: FromStr<Err = String> + Default,
    S: FromStr<Err = String> + Default,
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
                r"(?P<warp>[^: ]+)[Ww]arp(?: to (?P<spot>[^=]*$))?").unwrap();
            static ref GET: Regex = Regex::new(
                // Get
                // * Collect Station_Power from Antarctica > Power Room > Switch: Flip
                r"(?:\* )?(?:[Cc]ollect (?P<item>\w+) from|[Vv]isit) (?P<loc>[^=]*)(?: ==> (?P<dest>[^=]*$))?").unwrap();
            static ref MOVE: Regex = Regex::new(
                // Move
                //   Move... to Antarctica > West > Shed Entry ==> Shed > Interior (1)
                r"(?:[Mm]ove(?:\.\.\.)? to |Take exit )(?P<exit>[^=]* ==> [^=]*$)").unwrap();
            // TODO: remove
            static ref MOVE_GET: Regex = Regex::new(
                // MoveGet
                // * Take hybrid exit Glacier > The Big Drop > Water Surface: Drown, collecting Amashilama
                r"(?:\* )?Take hybrid exit (?P<loc>[^=]*)(?:, collecting (?P<item>\w+$))").unwrap();
            static ref MOVE_LOCAL: Regex = Regex::new(
                // MoveLocal
                //   Move... to Antarctica > Power Room > Switch
                r"[Mm]ove(?:\.\.\.)? to (?P<spot>[^=]*$)").unwrap();
            static ref ACTIVATE: Regex = Regex::new(
                // Activate
                // ! Do Amagi > Main Area > Carving: Key Combo
                r"(?:! )? (?:[Dd]o|[Aa]ctivate) (?P<action>.*$)").unwrap();
        }
        if let Some(cap) = GET.captures(s) {
            let item = extract_match(&cap, "item", s).unwrap_or_default();
            let loc = extract_match(&cap, "loc", s)?;
            // don't care about the dest spot for now
            Ok(History::G(
                <I as FromStr>::from_str(item).unwrap_or_default(),
                <L as FromStr>::from_str(loc)?,
            ))
        } else if let Some(cap) = MOVE.captures(s) {
            let exit = extract_match(&cap, "exit", s)?;
            Ok(History::E(<E as FromStr>::from_str(exit)?))
        } else if let Some(cap) = MOVE_GET.captures(s) {
            let loc = extract_match(&cap, "loc", s)?;
            let item = extract_match(&cap, "item", s).unwrap_or_default();
            Ok(History::G(
                <I as FromStr>::from_str(item).unwrap_or_default(),
                <L as FromStr>::from_str(loc)?,
            ))
        } else if let Some(cap) = MOVE_LOCAL.captures(s) {
            let spot = extract_match(&cap, "spot", s)?;
            Ok(History::L(<S as FromStr>::from_str(spot)?))
        } else if let Some(cap) = ACTIVATE.captures(s) {
            let action = extract_match(&cap, "action", s)?;
            Ok(History::A(<A as FromStr>::from_str(action)?))
        } else if let Some(cap) = WARP.captures(s) {
            let warp = extract_match(&cap, "warp", s)?;

            Ok(History::W(
                <Wp as FromStr>::from_str(warp)?,
                match extract_match(&cap, "spot", s) {
                    Ok(spot) => <S as FromStr>::from_str(spot)?,
                    Err(_) => S::default(),
                },
            ))
        } else if let Ok(spot) = <S as FromStr>::from_str(s) {
            Ok(History::L(spot))
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

pub trait Wrapper<T> {
    fn get(&self) -> &T;
    fn elapsed(&self) -> u32;
    fn time_since_visit(&self) -> u32;
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseContextWrapper<T, I, S, L, E, A, Wp> {
    ctx: T,
    elapsed: u32,
    time_since_visit: u32,
    hist_dur: u32,
    hist: Vec<History<I, S, L, E, A, Wp>>,
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
    fn time_since_visit(&self) -> u32 {
        self.time_since_visit
    }
}

impl<T: Ctx> ContextWrapper<T> {
    pub fn new(ctx: T) -> ContextWrapper<T> {
        ContextWrapper {
            ctx,
            elapsed: 0,
            time_since_visit: 0,
            hist_dur: 0,
            hist: Vec::new(),
        }
    }

    pub fn into_inner(self) -> T {
        self.ctx
    }

    pub fn to_solution(&self) -> Arc<Solution<T>> {
        Arc::new(Solution {
            elapsed: self.elapsed(),
            history: self.hist.clone(),
        })
    }

    pub fn into_solution(self) -> Arc<Solution<T>> {
        Arc::new(Solution {
            elapsed: self.elapsed,
            history: self.hist,
        })
    }

    pub fn with_times(ctx: T, elapsed: u32, time_since_visit: u32) -> ContextWrapper<T> {
        ContextWrapper {
            ctx,
            elapsed,
            time_since_visit,
            hist_dur: 0,
            hist: Vec::new(),
        }
    }

    pub fn append_history(&mut self, step: HistoryAlias<T>, dur: u32) {
        self.hist.push(step);
        self.hist_dur += dur;
    }

    pub fn recent_history(&self) -> &[HistoryAlias<T>] {
        &self.hist
    }

    pub fn recent_dur(&self) -> u32 {
        self.hist_dur
    }

    pub fn remove_history(&mut self) -> (Vec<HistoryAlias<T>>, u32) {
        let r = self.hist.clone();
        self.hist = Vec::new();
        let hist_dur = self.hist_dur;
        self.hist_dur = 0;
        (r, hist_dur)
    }

    fn elapse(&mut self, t: u32) {
        self.elapsed += t;
        self.time_since_visit += t;
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
        let dur = loc.time(&self.ctx, world);
        self.ctx.visit(loc.id());
        self.ctx.collect(loc.item(), world);
        self.ctx.spend(loc.price());
        if loc.dest() != Default::default() {
            self.ctx.set_position(loc.dest(), world);
            self.append_history(History::V(loc.item(), loc.id(), loc.dest()), dur);
        } else {
            self.append_history(History::G(loc.item(), loc.id()), dur);
        }
        self.elapse(dur);
        self.time_since_visit = 0;
    }

    pub fn exit<W, E>(&mut self, world: &W, exit: &E)
    where
        W: World<Exit = E>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    {
        let dur = exit.time(&self.ctx, world);
        self.ctx.set_position(exit.dest(), world);
        self.elapse(dur);
        self.ctx.spend(exit.price());
        self.append_history(History::E(exit.id()), dur);
    }

    pub fn move_local<W, E>(&mut self, world: &W, spot: E::SpotId, time: u32)
    where
        W: World<Exit = E>,
        T: Ctx<World = W>,
        E: Exit<Context = T>,
    {
        self.ctx.set_position(spot, world);
        self.elapse(time);
        self.append_history(History::L(spot), time);
    }

    pub fn move_condensed_edge<W, E>(
        &mut self,
        world: &W,
        edge: &CondensedEdge<T, E::SpotId, E::ExitId>,
    ) where
        W: World<Exit = E>,
        T: Ctx<World = W>,
        E: Exit<Context = T>,
        W::Location: Location<Context = T>,
    {
        self.ctx.set_position(edge.dst, world);
        let time = edge.time(world, self.get());
        self.elapse(time);
        self.append_history(History::C(edge.dst, edge.index), time);
    }

    pub fn warp<W, E, Wp>(&mut self, world: &W, warp: &Wp)
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
        let dur = warp.time(&self.ctx, world);
        warp.prewarp(&mut self.ctx, world);
        let dest = warp.dest(&self.ctx, world);
        assert!(
            dest != <E as Exit>::SpotId::default(),
            "Warp can't lead to SpotId::None: {}",
            warp.id()
        );
        self.ctx.set_position(dest, world);
        self.elapse(dur);
        self.ctx.spend(warp.price());
        warp.postwarp(&mut self.ctx, world);
        if warp.should_reload() {
            self.ctx.reload_game(world);
        }
        self.append_history(History::W(warp.id(), dest), dur);
    }

    pub fn activate<W, A>(&mut self, world: &W, action: &A)
    where
        W: World<Action = A>,
        T: Ctx<World = W>,
        A: Action<Context = T, Currency = <W::Location as Accessible>::Currency>,
    {
        let dur = action.time(&self.ctx, world);
        action.perform(&mut self.ctx, world);
        self.elapse(dur);
        self.ctx.spend(action.price());
        self.append_history(History::A(action.id()), dur);
    }

    pub fn can_replay<W, L, E, Wp>(&self, world: &W, step: HistoryAlias<T>) -> bool
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        match step {
            History::W(wp, dest) => {
                let warp = world.get_warp(wp);
                (dest == Default::default() || warp.dest(&self.ctx, world) == dest)
                    && warp.can_access(&self.ctx, world)
            }
            History::G(item, loc_id) | History::V(item, loc_id, ..) => {
                let spot_id = world.get_location_spot(loc_id);
                let loc = world.get_location(loc_id);
                spot_id == self.ctx.position()
                    && loc.item() == item
                    && !self.ctx.visited(loc.id())
                    && loc.can_access(&self.ctx, world)
            }
            History::E(exit_id) => {
                let spot_id = world.get_exit_spot(exit_id);
                let exit = world.get_exit(exit_id);
                spot_id == self.ctx.position() && exit.can_access(&self.ctx, world)
            }
            History::L(spot_id) => {
                let movement_state = self.ctx.get_movement_state(world);
                let (best_free, best_mvmts) = W::best_movements(self.ctx.position(), spot_id);
                self.ctx.position() != spot_id
                    && W::same_area(self.ctx.position(), spot_id)
                    && (best_free.is_some()
                        || best_mvmts
                            .into_iter()
                            .any(|(m, _)| T::is_subset(m, movement_state)))
            }
            History::A(act_id) => {
                let spot_id = world.get_action_spot(act_id);
                let action = world.get_action(act_id);
                (world.is_global_action(act_id) || self.ctx.position() == spot_id)
                    && action.can_access(&self.ctx, world)
            }
            History::C(spot_id, idx) => {
                let movement_state = self.ctx.get_movement_state(world);
                let edges = world.get_condensed_edges_from(self.ctx.position());
                idx < edges.len()
                    && edges[idx].dst == spot_id
                    && edges[idx].can_access(world, &self.ctx, movement_state)
            }
        }
    }

    pub fn replay<W, L, E, Wp>(&mut self, world: &W, step: HistoryAlias<T>)
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        // We skip checking validity ahead of time, i.e. can_access.
        // Some other times we should still assert some possibility.
        match step {
            History::W(wp, dest) => {
                self.warp(world, world.get_warp(wp));
                if dest != Default::default() {
                    assert!(
                        self.get().position() == dest,
                        "Invalid replay: warp {:?}",
                        wp
                    );
                }
            }
            History::G(item, loc_id) | History::V(item, loc_id, ..) => {
                let loc = world.get_location(loc_id);
                // We assert that if a loc is skippable that its item is never checked in any rule in this world+settings.
                if !loc.skippable() {
                    self.visit(world, loc);
                }
                assert!(loc.item() == item, "Invalid replay: visit {:?}", loc_id);
            }
            History::E(exit_id) => {
                let exit = world.get_exit(exit_id);
                self.exit(world, exit);
            }
            History::L(spot) => {
                let movement_state = self.ctx.get_movement_state(world);
                let time = self.ctx.local_travel_time(movement_state, spot);
                assert!(time != u32::MAX, "Invalid replay: move-local {:?}", spot);
                self.move_local(world, spot, time);
            }
            History::A(act_id) => {
                let action = world.get_action(act_id);
                self.activate(world, action);
            }
            History::C(spot, idx) => {
                let vce = world.get_condensed_edges_from(self.ctx.position());
                // Find the minimum of these edges that goes to spot that we can take
                // The list is pre-sorted in ascending order (not including penalties), so we can just take the first one.
                assert!(
                    idx < vce.len(),
                    "Invalid replay: move-condensed {:?} index={} len={})",
                    spot,
                    idx,
                    vce.len()
                );
                let ce = &vce[idx];
                assert!(
                    ce.dst == spot
                        && ce.can_access(world, self.get(), self.ctx.get_movement_state(world)),
                    "Invalid replay: move-condensed {:?}",
                    spot
                );
                self.move_condensed_edge(world, ce);
            }
        }
    }

    pub fn explain_pre_replay<W, L, E, Wp>(&self, world: &W, step: HistoryAlias<T>) -> String
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        match step {
            History::W(wp, _) => world.get_warp(wp).explain(self.get(), world),
            History::G(item_id, loc_id) | History::V(item_id, loc_id, _) => {
                let loc = world.get_location(loc_id);
                let e = loc.explain(self.get(), world);
                if item_id != loc.item() {
                    format!("{}\nItem does not match: {}", e, loc.item())
                } else {
                    e
                }
            }
            History::E(exit_id) => world.get_exit(exit_id).explain(self.get(), world),
            History::L(spot) => {
                let movement_state = self.ctx.get_movement_state(world);
                let time = self.ctx.local_travel_time(movement_state, spot);
                if time == u32::MAX {
                    format!(
                        "move-local to {} can't be done with movement state {:?}",
                        spot, movement_state
                    )
                } else {
                    format!(
                        "move-local to {} in {}ms with movement state {:?}",
                        spot, time, movement_state
                    )
                }
            }
            History::A(act_id) => world.get_action(act_id).explain(self.get(), world),
            History::C(spot_id, idx) => {
                let vce = world.get_condensed_edges_from(self.ctx.position());
                let mvs = self.ctx.get_movement_state(world);
                if idx >= vce.len() {
                    format!(
                        "Invalid CE index {} vs len {} at {}",
                        idx,
                        vce.len(),
                        self.ctx.position()
                    )
                } else if vce[idx].dst != spot_id {
                    format!(
                        "CE index {} is spot {} and not {}",
                        idx, vce[idx].dst, spot_id
                    )
                } else if !vce[idx].can_access(world, self.get(), mvs) {
                    vce[idx].explain(world, self.get(), mvs)
                } else {
                    String::from("")
                }
            }
        }
    }

    pub fn assert_and_replay<W, L, E, Wp>(&mut self, world: &W, step: HistoryAlias<T>)
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        assert!(
            self.can_replay(world, step),
            "can't replay \"{}\":\n{}",
            step,
            self.explain_pre_replay(world, step)
        );
        self.replay(world, step);
    }

    /// Checks whether the replay is possible. If it is, replays the step;
    /// otherwise returns an explanation of the properties evaluated.
    pub fn try_replay<W, L, E, Wp>(
        &mut self,
        world: &W,
        step: HistoryAlias<T>,
    ) -> Result<(), String>
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        if self.can_replay(world, step) {
            Ok(self.replay(world, step))
        } else {
            Err(self.explain_pre_replay(world, step))
        }
    }

    /// Returns the replay if all steps are valid in the order presented, or an error message if any step failed.
    /// This consumes the object, so use a clone if you want to keep a copy.
    pub fn try_replay_all<W, L, E, Wp>(
        mut self,
        world: &W,
        steps: &[HistoryAlias<T>],
    ) -> Result<Self, String>
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        for &step in steps {
            if let Err(s) = self.try_replay(world, step) {
                return Err(format!("Replay failed at \"{}\": {}", step, s));
            }
        }
        Ok(self)
    }

    /// Returns true and replays if the step is valid, or false with no changes otherwise.
    pub fn maybe_replay<W, L, E, Wp>(&mut self, world: &W, step: HistoryAlias<T>) -> bool
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        if self.can_replay(world, step) {
            self.replay(world, step);
            true
        } else {
            false
        }
    }

    /// Returns true and replays if all steps are valid in the order presented, or false if any step failed.
    /// This mutates the object, so use a clone if you want to keep a copy of the original.
    pub fn maybe_replay_all<W, L, E, Wp>(&mut self, world: &W, steps: &[HistoryAlias<T>]) -> bool
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        for &step in steps {
            if !self.maybe_replay(world, step) {
                return false;
            }
        }
        true
    }

    pub fn info(&self, est: u32, last: Option<HistoryAlias<T>>) -> String {
        format(format_args!(
            "At {}ms (elapsed={} est. left={}, since visit={}), visited={}\nNow: {} after {}",
            self.elapsed + est,
            self.elapsed,
            est,
            self.time_since_visit,
            self.get().count_visits(),
            self.ctx.position(),
            if let Some(val) = last {
                val.to_string()
            } else {
                String::from("None")
            },
        ))
    }
}

pub fn history_str<T, I>(history: I) -> String
where
    T: Ctx,
    I: Iterator<Item = HistoryAlias<T>>,
{
    let vec: Vec<String> = history.map(|h| h.to_string()).collect::<Vec<String>>();
    vec.join("\n")
}

pub fn history_preview<T, I>(history: I) -> String
where
    T: Ctx,
    I: Iterator<Item = HistoryAlias<T>>,
{
    let vec: Vec<String> = history
        .filter_map(|h| match h {
            History::G(..) | History::V(..) => Some(h.to_string()),
            _ => None,
        })
        .collect::<Vec<String>>();
    vec.join("\n")
}

pub fn history_summary<T, I>(history: I) -> String
where
    T: Ctx,
    I: Iterator<Item = HistoryAlias<T>>,
{
    let vec: Vec<String> = history
        .fold(Vec::new(), |mut v, h| {
            if let Some(lh) = v.last_mut() {
                match (*lh, h) {
                    (
                        History::E(..) | History::L(..) | History::C(..),
                        History::E(..) | History::L(..) | History::C(..),
                    ) => *lh = h,
                    _ => v.push(h),
                }
            } else {
                v.push(h);
            };
            v
        })
        .into_iter()
        .map(|h| match h {
            History::G(..) | History::A(..) | History::V(..) => h.to_string(),
            History::E(e) => format!("  Move... to {}", e),
            History::L(s) | History::C(s, ..) => {
                format!("  Move... to {}", s)
            }
            History::W(w, s) => {
                if s == Default::default() {
                    format!("  {}warp", w)
                } else {
                    format!("  {}warp to {}", w, s)
                }
            }
        })
        .collect();
    vec.join("\n")
}

pub fn collection_history<T, W, L, I>(history: I) -> impl Iterator<Item = HistoryAlias<T>>
where
    W: World<Location = L>,
    L: Location<Context = T>,
    T: Ctx<World = W>,
    I: Iterator<Item = HistoryAlias<T>>,
{
    history.filter(|h| match h {
        History::G(..) | History::V(..) => true,
        History::A(act_id) => W::action_has_visit(*act_id),
        _ => false,
    })
}

pub fn enumerated_collection_history<T, W, L, I>(
    history: I,
) -> impl Iterator<Item = (usize, HistoryAlias<T>)>
where
    W: World<Location = L>,
    L: Location<Context = T>,
    T: Ctx<World = W>,
    I: Iterator<Item = HistoryAlias<T>>,
{
    history.enumerate().filter(|h| match h.1 {
        History::G(..) | History::V(..) => true,
        History::A(act_id) => W::action_has_visit(act_id),
        _ => false,
    })
}

/// Produces an iterator of collection steps paired with the index range (inclusive) of all the steps
/// since the last collection.
pub fn collection_history_with_range_info<T, W, L, I>(
    history: I,
) -> impl Iterator<Item = (RangeInclusive<usize>, HistoryAlias<T>)>
where
    W: World<Location = L>,
    L: Location<Context = T>,
    T: Ctx<World = W>,
    I: Iterator<Item = HistoryAlias<T>>,
{
    let mut previ = 0;
    history.enumerate().filter_map(move |(i, h)| match h {
        History::G(..) | History::V(..) => {
            let r = previ..=i;
            previ = i + 1;
            Some((r, h))
        }
        History::A(act_id) => {
            if W::action_has_visit(act_id) {
                let r = previ..=i;
                previ = i + 1;
                Some((r, h))
            } else {
                None
            }
        }
        _ => None,
    })
}

pub fn history_to_full_series<T, W, L, E, Wp, I>(startctx: &T, world: &W, history: I) -> Vec<T>
where
    W: World<Location = L, Exit = E, Warp = Wp>,
    L: Location<Context = T>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <L as Accessible>::Currency>,
    Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    I: Iterator<Item = HistoryAlias<T>>,
{
    let mut vec = Vec::new();
    let mut prev = ContextWrapper::new(startctx.clone());
    for step in history {
        let mut next = prev.clone();
        next.assert_and_replay(world, step);
        vec.push(prev.into_inner());
        prev = next;
    }
    vec.push(prev.into_inner());
    vec
}
