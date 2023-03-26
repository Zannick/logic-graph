#![allow(unused)]

use analyzer::context::{ContextWrapper, Ctx, History};
use analyzer::world::*;
use analyzer::*;
use libaxiom_verge2::context::Context;
use libaxiom_verge2::graph::{self, *};
use libaxiom_verge2::items::Item;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

#[test]
fn test_name() {
    let mut world = graph::World::new();
    let mut ctx = Context::default();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_no_route!(
        &world,
        ctx,
        SpotId::Glacier__Vertical_Room__East_9,
        SpotId::Glacier__Vertical_Room__Peak
    );
}

#[test]
fn test_route() {
    let mut world = graph::World::new();
    let mut ctx = Context::default();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ledge_Grab);
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_this_route!(
        &world,
        ctx,
        SpotId::Glacier__Vertical_Room__Mid_9,
        vec![
            SpotId::Glacier__Vertical_Room__East_9,
            SpotId::Glacier__Vertical_Room__Mid_9,
            SpotId::Glacier__Vertical_Room__Peak,
        ]
    );
}

#[test]
fn test_obtain() {
    let mut world = graph::World::new();
    let mut ctx = Context::default();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.set_major_glitches(true);
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
    let mut ctx = Context::default();
    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    let mut ctx2 = ctx.clone();
    ctx2.set_major_glitches(true);
    expect_not_obtainable!(
        &world,
        ctx,
        SpotId::Glacier__Vertical_Room__East_9,
        Item::Ledge_Grab
    );
    expect_obtainable!(
        &world,
        ctx2,
        SpotId::Glacier__Vertical_Room__East_9,
        Item::Ledge_Grab
    );
}

#[test]
fn search() {
    let mut world = graph::World::new();
    let mut ctx = Context::default();
    ctx.energy = 300;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.save = SpotId::Glacier__Revival__Save_Point;
    ctx.visit(LocationId::Glacier__The_Big_Drop__Water_Surface__Drown);
    ctx.skip(LocationId::Glacier__Compass_Room__Center__Table);
    let verify = |c: &Context| {
        if c.has(Item::Boomerang) {
            Ok(())
        } else {
            Err("No boomerang")
        }
    };

    expect_eventually_requires_to_obtain!(
        &world,
        ctx,
        SpotId::Glacier__Vertical_Room__East_9,
        Item::Ledge_Grab,
        verify,
        2000
    );
}

fn serde_pass<T: Ctx>(ctx: &ContextWrapper<T>) -> ContextWrapper<T> {
    let mut buf = Vec::new();
    ctx.serialize(&mut Serializer::new(&mut buf)).unwrap();
    rmp_serde::from_slice(&buf).unwrap()
}

#[test]
fn asserde_true() {

    let mut world = graph::World::new();
    let mut ctx = Context::default();
    ctx.energy = 300;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.save = SpotId::Glacier__Revival__Save_Point;
    ctx.visit(LocationId::Glacier__The_Big_Drop__Water_Surface__Drown);
    ctx.skip(LocationId::Glacier__Compass_Room__Center__Table);

    let mut ctx = ContextWrapper::new(ctx);
    ctx.append_history(History::Get(Item::Amashilama, LocationId::Glacier__The_Big_Drop__Water_Surface__Drown));

    assert!(ctx == serde_pass(&ctx))
}
