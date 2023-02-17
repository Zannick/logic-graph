use crate::context::Ctx;
use std::fmt::Debug;
use std::hash::Hash;
use std::option::Option;

// type graph
// Context
// -> World
//    -> Location -> LocId, ExitId, CanonId
//       Accessible -> Context -> ItemId
//    -> Exit      --^    --^   -> SpotId
//       Accessible -> Context -> ItemId
//    -> Action -> ActionId
//       Accessible -> Context -> ItemId

pub trait Accessible {
    type Context: Ctx;
    fn can_access(&self, ctx: &Self::Context) -> bool;
}

pub trait Id: Copy + Clone + Debug + Eq + Hash + Ord + PartialOrd {}

pub trait Location: Accessible {
    type LocId: Id;
    type CanonId: Id;
    type ExitId: Id;
    type Currency: Id;

    fn id(&self) -> Self::LocId;
    fn item(&self) -> <Self::Context as Ctx>::ItemId;
    fn canon_id(&self) -> Self::CanonId;
    fn time(&self) -> i32;
    fn price(&self) -> &Self::Currency;
    fn exit_id(&self) -> &Option<Self::ExitId>;
}

pub trait Exit: Accessible {
    type ExitId: Id;
    type SpotId: Id;
    type LocId: Id;

    fn id(&self) -> Self::ExitId;
    fn dest(&self) -> Self::SpotId;
    fn connect(&mut self, dest: Self::SpotId);
    fn time(&self) -> i32;
    fn loc_id(&self) -> &Option<Self::LocId>;
}

pub trait Action: Accessible {
    type ActionId: Id;
    fn id(&self) -> Self::ActionId;
    fn time(&self) -> i32;
    fn perform(&self, ctx: &mut Self::Context);
}

pub trait Warp: Accessible {
    type WarpId: Id;
    type SpotId: Id;

    fn id(&self) -> Self::WarpId;
    fn dest(&self, ctx: &Self::Context) -> Self::SpotId;
    fn connect(&mut self, dest: Self::SpotId);
    fn time(&self) -> i32;
}

pub trait World {
    type Location: Location;
    type Exit: Exit<
        ExitId = <Self::Location as Location>::ExitId,
        LocId = <Self::Location as Location>::LocId,
        Context = <Self::Location as Accessible>::Context,
    >;
    type Action: Action<Context = <Self::Location as Accessible>::Context>;
    type Warp: Warp<
        Context = <Self::Location as Accessible>::Context,
        SpotId = <Self::Exit as Exit>::SpotId,
    >;

    fn get_location(&self, loc_id: <Self::Location as Location>::LocId) -> &Self::Location;
    fn get_exit(&self, ex_id: <Self::Exit as Exit>::ExitId) -> &Self::Exit;
    fn get_action(&self, act_id: <Self::Action as Action>::ActionId) -> &Self::Action;
    fn get_warp(&self, warp_id: <Self::Warp as Warp>::WarpId) -> &Self::Warp;

    fn get_spot_locations(&self, spot_id: <Self::Exit as Exit>::SpotId) -> &[Self::Location];
    fn get_spot_exits(&self, spot_id: <Self::Exit as Exit>::SpotId) -> &[Self::Exit];
    fn get_spot_actions(&self, spot_id: <Self::Exit as Exit>::SpotId) -> &[Self::Action];
    fn get_area_spots(
        &self,
        spot_id: <Self::Exit as Exit>::SpotId,
    ) -> &[<Self::Exit as Exit>::SpotId];
    fn get_warps(&self) -> &[Self::Warp];

    fn on_collect(
        &self,
        item: &<<Self::Location as Accessible>::Context as Ctx>::ItemId,
        ctx: &mut <Self::Location as Accessible>::Context,
    );
}
