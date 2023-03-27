#![allow(unused)]

use analyzer::context::{ContextWrapper, Ctx, History};
use analyzer::world::*;
use analyzer::*;
use libaxiom_verge2::context::Context;
use libaxiom_verge2::graph::{self, *};
use libaxiom_verge2::items::Item;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::io::Write;
use zstd::stream::read::Decoder;
use zstd::stream::write::Encoder;

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

fn serde_pass<T: Ctx>(ctx: &ContextWrapper<T>) -> Vec<u8> {
    let mut buf = Vec::new();
    ctx.serialize(&mut Serializer::new(&mut buf)).unwrap();
    buf
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
    ctx.append_history(History::Get(
        Item::Amashilama,
        LocationId::Glacier__The_Big_Drop__Water_Surface__Drown,
    ));

    let buf = serde_pass(&ctx);
    let json = serde_json::to_string(&ctx).unwrap();

    println!(
        "Serialized ctx ({} bytes) as json: {}\n as mp {} bytes: {:?}",
        std::mem::size_of::<ContextWrapper<Context>>(),
        json,
        buf.len(),
        buf
    );
    let ctx2 = rmp_serde::from_slice(&buf).unwrap();
    let ctx3 = serde_json::from_str(&json).unwrap();

    let mut cmp = vec![buf.len()];
    for i in 1..=21 {
        let vec = Vec::new();
        let mut zcmprsr = Encoder::new(vec, i).unwrap();
        zcmprsr.write(&buf);
        cmp.push(zcmprsr.finish().unwrap().len());
    }

    println!("Compression results: {:?}", cmp);

    assert!(ctx == ctx2);
    assert!(ctx == ctx3);
    assert!(ctx
        .get()
        .visited(LocationId::Glacier__The_Big_Drop__Water_Surface__Drown));
    assert!(ctx2
        .get()
        .visited(LocationId::Glacier__The_Big_Drop__Water_Surface__Drown));
    assert!(ctx3
        .get()
        .visited(LocationId::Glacier__The_Big_Drop__Water_Surface__Drown));
}
