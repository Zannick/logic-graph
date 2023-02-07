pub trait ItemContext {
    type ItemId;

    fn has(&self, item: &Self::ItemId) -> bool;
    fn count(&self, item: &Self::ItemId) -> i16;
    fn collect(&mut self, item: &Self::ItemId);
}

pub trait PosContext {
    type SpotId;

    fn position(&self) -> &Self::SpotId;
    fn set_position(&mut self, pos: &Self::SpotId);
}
