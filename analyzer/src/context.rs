use crate::world::*;

pub trait ItemContext<ItemId> {
    fn has(&self, item: &ItemId) -> bool;
    fn count(&self, item: &ItemId) -> i16;
}

pub trait PosContext<SpotId> {
    fn position(&self) -> &SpotId;
    fn set_position(&mut self, pos: SpotId);
}
