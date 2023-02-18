use crate::world::*;
use sort_by_derive::SortBy;
use std::fmt::{self, format, Debug, Display};

pub trait Ctx: Clone + Eq + Debug {
    type World: World;
    type ItemId: Id;
    type AreaId: Id;
    type RegionId: Id;

    fn has(&self, item: Self::ItemId) -> bool;
    fn count(&self, item: Self::ItemId) -> i16;
    fn collect(&mut self, item: Self::ItemId);

    fn position(&self) -> <<Self::World as World>::Exit as Exit>::SpotId;
    fn set_position(&mut self, pos: <<Self::World as World>::Exit as Exit>::SpotId);

    fn can_afford(&self, cost: &<<Self::World as World>::Location as Location>::Currency) -> bool;
    fn spend(&mut self, cost: &<<Self::World as World>::Location as Location>::Currency);

    fn visit(&mut self, loc_id: <<Self::World as World>::Location as Location>::LocId);
    fn skip(&mut self, loc_id: <<Self::World as World>::Location as Location>::LocId);
    fn todo(&self, loc_id: <<Self::World as World>::Location as Location>::LocId) -> bool;
    fn visited(&self, loc_id: <<Self::World as World>::Location as Location>::LocId) -> bool;
    fn skipped(&self, loc_id: <<Self::World as World>::Location as Location>::LocId) -> bool;

    fn all_spot_checks(&self, id: <<Self::World as World>::Exit as Exit>::SpotId) -> bool;
    fn all_area_checks(&self, id: Self::AreaId) -> bool;
    fn all_region_checks(&self, id: Self::RegionId) -> bool;

    fn local_travel_time(&self, dest: <<Self::World as World>::Exit as Exit>::SpotId) -> i32;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum History<T>
where
    T: Ctx,
{
    Warp(<<<T as Ctx>::World as World>::Warp as Warp>::WarpId),
    Get(
        <T as Ctx>::ItemId,
        <<<T as Ctx>::World as World>::Location as Location>::LocId,
    ),
    Move(<<<T as Ctx>::World as World>::Exit as Exit>::ExitId),
    MoveLocal(<<<T as Ctx>::World as World>::Exit as Exit>::SpotId),
    Activate(<<<T as Ctx>::World as World>::Action as Action>::ActionId),
}

impl<T> Display for History<T>
where
    T: Ctx,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            History::Warp(warp) => write!(f, "{}warp", warp),
            History::Get(item, loc) => write!(f, "got {} from {}", item, loc),
            History::Move(exit) => Display::fmt(&exit, f),
            History::MoveLocal(spot) => write!(f, "move to {}", spot),
            History::Activate(action) => write!(f, "do {}", action),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    None,
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
    pub history: Box<Vec<History<T>>>,
    pub lastmode: Mode,
}

impl<T: Ctx> ContextWrapper<T> {
    pub fn new(ctx: T) -> ContextWrapper<T> {
        ContextWrapper {
            ctx,
            elapsed: 0,
            history: Box::new(vec![]),
            lastmode: Mode::None,
        }
    }

    pub fn elapse(&mut self, t: i32) {
        self.elapsed += t;
    }

    pub fn elapsed(&self) -> i32 {
        self.elapsed
    }

    pub fn get(&self) -> &T {
        &self.ctx
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.ctx
    }

    pub fn info(&self) -> String {
        format(format_args!(
            "At {} after {}ms\n{} steps, last={}\nmode={:?}",
            self.ctx.position(),
            self.elapsed,
            self.history.len(),
            if let Some(val) = self.history.last() {
                val.to_string()
            } else {
                String::from("None")
            },
            self.lastmode
        ))
    }
}
