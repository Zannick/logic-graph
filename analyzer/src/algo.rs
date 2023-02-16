use crate::context::*;
use crate::world::*;
use std::collections::BinaryHeap;

pub fn do_the_thing(world: &impl World, ctx: impl Ctx) {
    let mut heap = BinaryHeap::new();
    let ctx = ContextWrapper::new(ctx);
    heap.push(&ctx);
    let mut ctx2 = ctx.clone();
    ctx2.elapse(15);
    heap.push(&ctx2);
    println!("first: {}", heap.pop().expect("manual").elapsed());
    println!("second: {}", heap.pop().expect("manual").elapsed());
}
