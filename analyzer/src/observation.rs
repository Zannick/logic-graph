use crate::matchertrie::{MatcherDispatch, Observable};
use crate::world::Id;
use std::fmt::Debug;

pub trait Observation<V>: Debug {
    type Ctx: Observable;
    type LocId: Id;
    type Matcher: MatcherDispatch<Value = V> + Default + Send + Sync + 'static;
}
