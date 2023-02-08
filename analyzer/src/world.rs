use crate::context::Ctx;

pub trait Location {
    type LocId;
    type CanonId;
    type Context: Ctx;

    fn id(&self) -> &Self::LocId;
    fn item(&self) -> &<Self::Context as Ctx>::ItemId;
    fn clear_item(&mut self);
    fn get_canon_id(&self) -> &Self::CanonId;

    fn can_access(&self, ctx: &Self::Context) -> bool;
    fn take(&mut self, ctx: &mut Self::Context);
}

pub trait Exit {
    type ExitId;
    type SpotId;
    type Context;

    fn id(&self) -> &Self::ExitId;
    fn dest(&self) -> &Self::SpotId;
    fn connect(&mut self, dest: &Self::SpotId);
    fn can_access(&self, ctx: &Self::Context) -> bool;
}

pub trait Hybrid {
    type ExitId;
    type CanonId;
    type Context: Ctx;

    fn id(&self) -> &Self::ExitId;
    fn item(&self) -> &<Self::Context as Ctx>::ItemId;
    fn clear_item(&mut self);
    fn get_canon_id(&self) -> &Self::CanonId;
    fn can_access(&self, ctx: &Self::Context) -> bool;
}

pub trait Action {
    type ActionId;
    type Context;
    fn id(&self) -> &Self::ActionId;
    fn can_access(&self, ctx: &Self::Context) -> bool;
}

pub trait Spot {
    type SpotId;
    type Location: Location;
    type Exit: Exit;
    type Action: Action;
    type Hybrid: Hybrid;

    fn id(&self) -> &Self::SpotId;
    fn get_coord(&self) -> (i16, i16);
    fn get_locations(&self) -> &[Self::Location];
    fn get_exits(&self) -> &[Self::Exit];
    fn get_actions(&self) -> &[Self::Action];
    fn get_hybrids(&self) -> &[Self::Hybrid];
}

pub trait Area {
    type AreaId;
    type Spot;
    fn id(&self) -> &Self::AreaId;
    fn get_spots(&self) -> &[Self::Spot];
}

pub trait Region {
    type RegionId;
    fn id(&self) -> &Self::RegionId;
}

pub trait World {
    type Location: Location;
    type Exit: Exit;
    //type Action: Action;
    //type Hybrid: Hybrid;
    type Spot; //: Spot;
               //type Area: Area;
               //type Region: Region;
    type Context: Ctx;

    fn get_location(&self, locid: &<Self::Location as Location>::LocId) -> &Self::Location;
    fn get_location_mut(
        &mut self,
        locid: &<Self::Location as Location>::LocId,
    ) -> &mut Self::Location;
}
