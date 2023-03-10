#![allow(unused)]

use analyzer::context::Ctx;
use analyzer::*;
use libaxiom_verge2::context::Context;
use libaxiom_verge2::graph::*;

#[test]
fn test_name() {
    let mut world = World::new();
    let mut ctx = Context::new();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.amashilama = true;
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_no_route!(
        &world,
        ctx,
        SpotId::Glacier__Vertical_Room_Top__East_9,
        SpotId::Glacier__Vertical_Room_Top__Peak
    );
}
