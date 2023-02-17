use crate::context::Ctx;
use std::fmt::Debug;
use std::hash::Hash;
use std::option::Option;

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

    fn id(&self) -> &Self::LocId;
    fn item(&self) -> &<Self::Context as Ctx>::ItemId;
    fn canon_id(&self) -> &Self::CanonId;
    fn time(&self) -> i32;
    fn price(&self) -> &Self::Currency;
    fn exit_id(&self) -> &Option<Self::ExitId>;

    // to be replaced with similar methods on Context
    //fn take(&self, ctx: &mut Self::Context);
    //fn skip(&self, ctx: &mut Self::Context);
}

pub trait Exit: Accessible {
    type ExitId: Id;
    type SpotId: Id;
    type LocId: Id;

    fn id(&self) -> &Self::ExitId;
    fn dest(&self) -> &Self::SpotId;
    fn connect(&mut self, dest: &Self::SpotId);
    fn time(&self) -> i32;
    fn loc_id(&self) -> &Option<Self::LocId>;
}

pub trait Action: Accessible {
    type ActionId: Id;
    fn id(&self) -> &Self::ActionId;
    fn time(&self) -> i32;
    fn perform(&self, ctx: &mut Self::Context);
}

pub trait Spot {
    type SpotId: Id;
    type Location: Location;
    type Exit: Exit;
    type Action: Action;

    fn id(&self) -> &Self::SpotId;
    // might not be necessary if we hardcode distances
    //fn get_coord(&self) -> (i16, i16);
    fn locations(&self) -> &[Self::Location];
    fn exits(&self) -> &[Self::Exit];
    fn actions(&self) -> &[Self::Action];
}

// This is necessary to handle movement calculations
pub trait Area {
    type AreaId: Id;
    type Spot: Spot;
    fn id(&self) -> &Self::AreaId;
    fn spots(&self) -> &[Self::Spot];
}

// This one might not be necessary.
pub trait Region {
    type RegionId;
    fn id(&self) -> &Self::RegionId;
}

pub trait World {
    type Context: Ctx<SpotId = Self::SpotId, World = Self>;
    type Location: Location + Accessible<Context = Self::Context>;
    type Exit: Exit<SpotId = Self::SpotId> + Accessible<Context = Self::Context>;
    type Action: Action;
    type SpotId: Id;
    //type AreaId;
    //type Spot: Spot<Location = Self::Location, Exit = Self::Exit>;
    //type Area: Area<Spot = Self::Spot>;
    //type Region: Region;

    fn get_location(&self, loc_id: <Self::Location as Location>::LocId) -> &Self::Location;
    fn get_exit(&self, ex_id: <Self::Exit as Exit>::ExitId) -> &Self::Exit;
    fn get_action(&self, act_id: <Self::Action as Action>::ActionId) -> &Self::Action;
    //fn get_spot(&self, sp_id: <Self::Spot as Spot>::SpotId) -> &Self::Spot;
    //fn get_area(&self, area_id: &<Self::Area as Area>::AreaId) -> &Self::Area;

    fn get_spot_locations(&self, spot_id: Self::SpotId) -> &[Self::Location];
    fn get_spot_exits(&self, spot_id: Self::SpotId) -> &[Self::Exit];
    fn get_spot_actions(&self, spot_id: Self::SpotId) -> &[Self::Action];
    fn get_area_spots(&self, spot_id: Self::SpotId) -> &[Self::SpotId];

    fn on_collect(
        &self,
        item: &<<Self::Location as Accessible>::Context as Ctx>::ItemId,
        ctx: &mut <Self::Location as Accessible>::Context,
    );
}
