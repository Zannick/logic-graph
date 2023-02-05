
pub trait Location<LocId, ItemId>
{
    fn id(&self) -> &LocId;
    fn get_item(&self) -> &ItemId;
    fn clear_item(&mut self);
}

pub trait Spot<SpotId, LocId, ItemId>
{
    fn id(&self) -> &SpotId;
}

pub trait World<LocId, Loc, ItemId>
where
    Loc: Location<LocId, ItemId>,
{
    fn get_location(&self, locid: &LocId) -> &mut Loc;
}
