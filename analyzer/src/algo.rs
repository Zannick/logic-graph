use crate::access::*;
use crate::context::*;
use crate::world::*;
use std::collections::BinaryHeap;

pub fn do_the_thing<T, S>(world: &impl World<Context = T, SpotId = S>, ctx: T)
where
    T: Ctx<SpotId = S>,
    S: Id,
{
    let mut heap = BinaryHeap::new();
    let ctx = ContextWrapper::new(ctx);
    heap.push(&ctx);
    let spot_map = access(world, &ctx);
    println!("{:#?}", spot_map);
}
