use crate::world::World;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Ctx: Clone + Eq {
    // This could be distilled way down into just, World: World.
    // But that would require a lot of boilerplate typing in the functions below.
    type World: World<Context = Self, SpotId = Self::SpotId>;
    type ItemId: Copy + Clone + Debug + Eq + Hash + Ord + PartialOrd;
    type LocationId: Copy + Clone + Debug + Eq + Hash + Ord + PartialOrd;
    type SpotId: Copy + Clone + Debug + Eq + Hash + Ord + PartialOrd;
    type AreaId: Copy + Clone + Debug + Eq + Hash + Ord + PartialOrd;
    type RegionId: Copy + Clone + Debug + Eq + Hash + Ord + PartialOrd;
    type ActionId: Copy + Clone + Debug + Eq + Hash + Ord + PartialOrd;
    type ExitId: Copy + Clone + Debug + Eq + Hash + Ord + PartialOrd;
    type Currency: Copy + Clone + Debug + Eq + Hash + Ord + PartialOrd;

    fn has(&self, item: Self::ItemId) -> bool;
    fn count(&self, item: Self::ItemId) -> i16;
    fn collect(&mut self, item: Self::ItemId);

    fn position(&self) -> Self::SpotId;
    fn set_position(&mut self, pos: Self::SpotId);

    fn can_afford(&self, cost: &Self::Currency) -> bool;
    fn spend(&mut self, cost: &Self::Currency);

    fn visit(&mut self, loc_id: Self::LocationId);
    fn skip(&mut self, loc_id: Self::LocationId);

    fn all_spot_checks(&self, id: Self::SpotId) -> bool;
    fn all_area_checks(&self, id: Self::AreaId) -> bool;
    fn all_region_checks(&self, id: Self::RegionId) -> bool;

    fn local_travel_time_to(&self, id: Self::SpotId) -> i32;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum History<T>
where
    T: Ctx,
{
    Warp(<T as Ctx>::SpotId),
    Get(<T as Ctx>::LocationId),
    Move(<T as Ctx>::ExitId),
    MoveLocal(<T as Ctx>::SpotId),
    Activate(<T as Ctx>::ActionId),
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    None,
    Explore,
    Check,
    Activate,
}

#[derive(Clone, Debug, Eq)]
pub struct ContextWrapper<T>
where
    T: Ctx,
{
    ctx: T,
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
}

impl<T: Ctx> Ord for ContextWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.elapsed.cmp(&self.elapsed)
    }
}
impl<T: Ctx> PartialOrd for ContextWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
impl<T: Ctx> PartialEq for ContextWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.elapsed == other.elapsed
    }
}
