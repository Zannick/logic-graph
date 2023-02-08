pub trait Ctx {
    type ItemId;
    type SpotId;
    type Currency;

    fn has(&self, item: &Self::ItemId) -> bool;
    fn count(&self, item: &Self::ItemId) -> i16;
    fn collect(&mut self, item: &Self::ItemId);

    fn position(&self) -> &Self::SpotId;
    fn set_position(&mut self, pos: &Self::SpotId);

    fn can_afford(&self, cost: &Self::Currency) -> bool;
    fn spend(&mut self, cost: &Self::Currency);
}
