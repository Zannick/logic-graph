#![allow(unused)]

use analyzer::context::Ctx;
use analyzer::world::*;
use analyzer::*;
use libaxiom_verge2::context::Context;
use libaxiom_verge2::graph::{self, *};
use libaxiom_verge2::items::Item;

#[test]
fn test_name() {
    let mut world = graph::World::new();
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

#[test]
fn test_route() {
    let mut world = graph::World::new();
    let mut ctx = Context::new();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.amashilama = true;
    ctx.ledge_grab = true;
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_this_route!(
        &world,
        ctx,
        SpotId::Glacier__Vertical_Room_Top__Mid_9,
        vec![
            SpotId::Glacier__Vertical_Room_Top__East_9,
            SpotId::Glacier__Vertical_Room_Top__Mid_9,
            SpotId::Glacier__Vertical_Room_Top__Peak,
        ]
    );
}

#[test]
fn test_obtain() {
    let mut world = graph::World::new();
    let mut ctx = Context::new();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.amashilama = true;
    ctx.ice_axe = true;
    ctx.major_glitches = true;
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_obtainable!(
        &world,
        ctx,
        SpotId::Glacier__Revival__Save_Point,
        Item::Ledge_Grab
    );
}

#[test]
fn test_require() {
    let mut world = graph::World::new();
    let mut ctx = Context::new();
    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.amashilama = true;
    ctx.ice_axe = true;
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    let mut ctx2 = ctx.clone();
    ctx2.major_glitches = true;
    expect_not_obtainable!(
        &world,
        ctx,
        SpotId::Glacier__Vertical_Room_Top__East_9,
        Item::Ledge_Grab
    );
    expect_obtainable!(
        &world,
        ctx2,
        SpotId::Glacier__Vertical_Room_Top__East_9,
        Item::Ledge_Grab
    );
}

#[test]
fn search() {
    let mut world = graph::World::new();
    let mut ctx = Context::new();
    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.amashilama = true;
    ctx.ice_axe = true;
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_eventually_requires!(
        &world,
        ctx,
        SpotId::Glacier__Vertical_Room_Top__East_9,
        Item::Ledge_Grab,
        Item::Boomerang
    );
}
