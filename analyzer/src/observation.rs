

use crate::matchertrie::{MatcherDispatch, Observable};
use crate::world::Id;
use std::fmt::Debug;

pub trait Observation: Debug {
    type Ctx: Observable;
    type LocId: Id;
    type Matcher: MatcherDispatch + Default + Send + Sync + 'static;
}
