#![allow(unused)]

use analyzer::context::{ContextWrapper, Ctx, History, Wrapper};
use analyzer::world::*;
use analyzer::*;
use libaxiom_verge2::context::Context;
use libaxiom_verge2::graph::{self, *};
use libaxiom_verge2::graph_enums::*;
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
    world.condense_graph();
    let mut ctx = Context::default();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_no_route!(
        &*world,
        ctx,
        Context,
        SpotId::Glacier__Vertical_Room__East_9,
        SpotId::Glacier__Vertical_Room__Peak
    );
}

#[test]
fn test_route() {
    let mut world = graph::World::new();
    world.condense_graph();
    let mut ctx = Context::default();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ledge_Grab);
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_this_route!(
        &*world,
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
    world.condense_graph();
    let mut ctx = Context::default();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    world.major_glitches = true;
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_obtainable!(
        &*world,
        ctx,
        SpotId::Glacier__Revival__Save_Point,
        Item::Ledge_Grab
    );
}

#[test]
fn test_require() {
    let mut world = graph::World::new();
    world.condense_graph();
    let mut ctx = Context::default();
    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_not_obtainable!(
        &*world,
        ctx.clone(),
        Context,
        SpotId::Glacier__Vertical_Room__East_9,
        Item::Ledge_Grab
    );

    world.major_glitches = true;
    expect_obtainable!(
        &*world,
        ctx,
        SpotId::Glacier__Vertical_Room__East_9,
        Item::Ledge_Grab
    );
}

#[ignore]
#[test]
fn search() {
    let mut world = graph::World::new();
    world.condense_graph();
    let mut ctx = Context::default();
    ctx.energy = 300;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.save = SpotId::Glacier__Revival__Save_Point;
    ctx.visit(LocationId::Glacier__Sea_Burial__Collapsing_Ceiling__Drown);
    let verify = |c: &Context| {
        if c.has(Item::Boomerang) {
            Ok(())
        } else {
            Err("No boomerang")
        }
    };

    expect_eventually_requires_to_obtain!(
        &*world,
        ctx,
        Context,
        SpotId::Glacier__Vertical_Room__East_9,
        Item::Ledge_Grab,
        verify,
        500
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
    world.condense_graph();
    let mut ctx = Context::default();
    ctx.energy = 300;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.save = SpotId::Glacier__Revival__Save_Point;
    ctx.visit(LocationId::Glacier__Sea_Burial__Collapsing_Ceiling__Drown);

    let mut ctx = ContextWrapper::new(ctx);
    ctx.append_history(History::G(
        Item::Amashilama,
        LocationId::Glacier__Sea_Burial__Collapsing_Ceiling__Drown,
    ), 20);

    let buf = serde_pass(&ctx);
    let json = serde_json::to_string(&ctx).unwrap();

    let mut buf2 = Vec::new();
    ctx.get().serialize(&mut Serializer::new(&mut buf2)).unwrap();
    println!(
        "Serialized Context ({} bytes) as mp: {} bytes",
        std::mem::size_of::<Context>(),
        buf2.len()
    );

    println!(
        "Serialized ctxwrapper ({} bytes) as json: {}\n as mp {} bytes: {:?}",
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
        zcmprsr.write(&buf).unwrap();
        cmp.push(zcmprsr.finish().unwrap().len());
    }

    println!("Compression results: {:?}", cmp);

    assert!(ctx == ctx2);
    assert!(ctx == ctx3);
    assert!(ctx
        .get()
        .visited(LocationId::Glacier__Sea_Burial__Collapsing_Ceiling__Drown));
    assert!(ctx2
        .get()
        .visited(LocationId::Glacier__Sea_Burial__Collapsing_Ceiling__Drown));
    assert!(ctx3
        .get()
        .visited(LocationId::Glacier__Sea_Burial__Collapsing_Ceiling__Drown));
}
