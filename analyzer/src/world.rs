use crate::context::Ctx;

pub trait Accessible {
    type Context: Ctx;
    fn can_access(&self, ctx: &Self::Context) -> bool;
}

pub trait Location: Accessible {
    type LocId;
    type CanonId;

    fn id(&self) -> &Self::LocId;
    fn item(&self) -> &<Self::Context as Ctx>::ItemId;
    fn canon_id(&self) -> &Self::CanonId;

    // to be replaced with similar methods on Context
    //fn take(&self, ctx: &mut Self::Context);
    //fn skip(&self, ctx: &mut Self::Context);
}

pub trait Exit: Accessible {
    type ExitId;
    type SpotId;

    fn id(&self) -> &Self::ExitId;
    fn dest(&self) -> &Self::SpotId;
    fn connect(&mut self, dest: &Self::SpotId);
}

pub trait Action: Accessible {
    type ActionId;
    fn id(&self) -> &Self::ActionId;
}

pub trait Spot {
    type SpotId;
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
    type AreaId;
    type Spot;
    fn id(&self) -> &Self::AreaId;
    fn spots(&self) -> &[Self::Spot];
}

// This one might not be necessary.
pub trait Region {
    type RegionId;
    fn id(&self) -> &Self::RegionId;
}

pub trait World {
    type Location: Location;
    type Exit: Exit;
    type Action: Action;
    type Spot: Spot;
    //type Area: Area;
    //type Region: Region;

    fn get_location(&self, loc_id: &<Self::Location as Location>::LocId) -> &Self::Location;
    fn get_exit(&self, ex_id: &<Self::Exit as Exit>::ExitId) -> &Self::Exit;
    fn get_action(&self, act_id: &<Self::Action as Action>::ActionId) -> &Self::Action;
    fn get_spot(&self, sp_id: &<Self::Spot as Spot>::SpotId) -> &Self::Spot;
}