pub trait Location {
    type LocId;
    type ItemId;
    type CanonId;
    fn id(&self) -> &Self::LocId;
    fn get_item(&self) -> &Self::ItemId;
    fn clear_item(&mut self);
    fn get_canon_id(&self) -> &Self::CanonId;
}

pub trait Exit {
    type ExitId;
    type Spot: Spot;
    fn id(&self) -> &Self::ExitId;
    fn src(&self) -> &Self::Spot;
    fn dest(&self) -> &Self::Spot;
}

pub trait Hybrid {
    type ExitId;
    type ItemId;
    type CanonId;

    fn id(&self) -> &Self::ExitId;
    fn get_item(&self) -> &Self::ItemId;
    fn clear_item(&mut self);
    fn get_canon_id(&self) -> &Self::CanonId;
}

pub trait Action {
    type ActionId;
    fn id(&self) -> &Self::ActionId;
}

pub trait Spot {
    type SpotId;
    type Location: Location;
    type Exit: Exit;
    type Action: Action;
    type Hybrid: Hybrid;

    fn id(&self) -> &Self::SpotId;
    fn get_coord(&self) -> (i16, i16);
    fn get_locations(&self) -> &Vec<Self::Location>;
    fn get_exits(&self) -> &Vec<Self::Exit>;
    fn get_actions(&self) -> &Vec<Self::Action>;
    fn get_hybrids(&self) -> &Vec<Self::Hybrid>;
}

pub trait Area {
    type AreaId;
    type Spot;
    fn id(&self) -> &Self::AreaId;
    fn get_spots(&self) -> &Vec<Self::Spot>;
}

pub trait Region {
    type RegionId;
    fn id(&self) -> &Self::RegionId;
}

pub trait World {
    type ItemId;
    type Location: Location;
    type Exit: Exit;
    type Action: Action;
    type Hybrid: Hybrid;
    type Spot: Spot;
    type Area: Area;
    type Region: Region;

    fn get_location(&self, locid: &<Self::Location as Location>::LocId) -> &mut Self::Location;
}
