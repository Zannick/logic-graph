#![allow(unused)]

use analyzer::access::move_to;
use analyzer::context::{ContextWrapper, Ctx, History, Wrapper};
use analyzer::steiner::{build_simple_graph, EdgeId, NodeId, ShortestPaths, SteinerAlgo};
use analyzer::world::{Accessible, Action, Exit, Location, Warp, World as _};
use analyzer::*;
use lazy_static::lazy_static;
use libaxiom_verge2::context::{enums, Context};
use libaxiom_verge2::graph::*;
use libaxiom_verge2::items::Item;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::io::Write;
use zstd::stream::read::Decoder;
use zstd::stream::write::Encoder;

lazy_static! {
    static ref WORLD: Box<World> = {
        let mut world = World::new();
        world.condense_graph();
        world
    };
    static ref SPATHS: ShortestPaths<NodeId<World>, EdgeId<World>> =
        ShortestPaths::from_graph(build_simple_graph(&**WORLD, &Context::default()));
}

#[test]
fn test_name() {
    let mut ctx = Context::default();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_no_route!(
        &**WORLD,
        ctx,
        Context,
        SpotId::Glacier__Vertical_Room__East_9,
        SpotId::Glacier__Vertical_Room__Peak
    );
}

#[test]
fn test_route() {
    let mut ctx = Context::default();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ledge_Grab);
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    expect_this_route!(
        &**WORLD,
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
    let mut ctx = Context::default();

    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    let mut world = WORLD.clone();
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
    let mut ctx = Context::default();
    ctx.energy = 30;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.save = SpotId::Glacier__Revival__Save_Point;

    let mut world = WORLD.clone();
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

#[test]
fn test_penalty() {
    let mut ctx = Context::default();
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Anuman);
    ctx.add_item(Item::Remote_Drone);
    ctx.add_item(Item::Breach_Attractor);
    ctx.save = SpotId::Glacier__Revival__Save_Point;
    ctx.set_position(SpotId::Amagi__East_Lake__Arch_West, &**WORLD);
    ctx.portal = SpotId::Amagi__East_Lake__Portal_Stand;
    let mut wrapper = ContextWrapper::new(ctx);

    let action = *WORLD.get_action(ActionId::Global__Move_Portal_Here);
    assert!(
        action.time(wrapper.get(), &**WORLD) > action.base_time(),
        "Penalty not calculated for portal attract"
    );
    assert!(
        action.can_access(wrapper.get(), &**WORLD),
        "{}",
        action.explain(wrapper.get(), &**WORLD)
    );
}

#[test]
fn test_move_to() {
    let mut ctx = Context::default();
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Underwater_Movement);
    ctx.add_item(Item::Remote_Drone);
    ctx.add_item(Item::Slingshot_Hook);
    ctx.add_item(Item::Infect);
    ctx.add_item(Item::Nanite_Mist);
    ctx.save = SpotId::Glacier__Revival__Save_Point;
    ctx.breach_save = SpotId::Amagi_Breach__East_Ruins__Save_Point;
    ctx.mode = enums::Mode::Drone;
    ctx.energy = 300;
    ctx.set_position(
        SpotId::Amagi_Breach__East_Ruins__Northeast_Bubbles_Center,
        &**WORLD,
    );

    let movement_state = ctx.get_movement_state(&**WORLD);
    let ltt = ctx.local_travel_time(
        movement_state,
        SpotId::Amagi_Breach__East_Ruins__Northeast_Bubbles_Corner_Access,
    );
    let exit1 = *WORLD.get_exit(ExitId::Amagi_Breach__East_Ruins__Northeast_Bubbles_Center__ex__Northeast_Bubbles_Corner_Access_1);
    let exit2 = *WORLD.get_exit(ExitId::Amagi_Breach__East_Ruins__Northeast_Bubbles_Center__ex__Northeast_Bubbles_Corner_Access_2);
    assert!(exit1.can_access(&ctx, &**WORLD));
    assert!(!exit2.can_access(&ctx, &**WORLD));
    let e1time = exit1.time(&ctx, &**WORLD);
    assert!(e1time > ltt);
    let wrapper = ContextWrapper::new(ctx.clone());
    let result = move_to(
        &**WORLD,
        wrapper,
        SpotId::Amagi_Breach__East_Ruins__Northeast_Bubbles_Corner_Access,
        &*SPATHS,
    ).expect("didn't reach dest");
    assert!(
        result.elapsed() == ltt,
        "Expected move_to to reach dest in {} but got there in {}",
        ltt,
        result.elapsed()
    );

    ctx.set_position(SpotId::Amagi_Breach__East_Ruins__High_Rock_Lower_Ledge, &**WORLD);
    let ltt2 = ctx.local_travel_time(movement_state, SpotId::Amagi_Breach__East_Ruins__Northeast_Bubbles_Center);
    ctx.set_position(SpotId::Amagi_Breach__East_Ruins__Arch_West, &**WORLD);
    let exit3 = *WORLD.get_exit(ExitId::Amagi_Breach__East_Ruins__Arch_West__ex__High_Rock_Lower_Ledge_1);
    assert!(exit3.can_access(&ctx, &**WORLD));
    let e3time = exit3.time(&ctx, &**WORLD);

    let wrapper = ContextWrapper::new(ctx);
    let result = move_to(
        &**WORLD,
        wrapper,
        SpotId::Amagi_Breach__East_Ruins__Northeast_Bubbles_Corner_Access,
        &*SPATHS,
    ).expect("didn't reach dest");
    assert!(
        result.elapsed() == ltt + ltt2 + e3time,
        "Expected move_to to reach dest in {} but got there in {} ({:?})",
        ltt + ltt2 + e3time,
        result.elapsed(),
        result.recent_history(),
    );
}

#[ignore]
#[test]
fn search() {
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
        &**WORLD,
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
    let mut ctx = Context::default();
    ctx.energy = 300;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.save = SpotId::Glacier__Revival__Save_Point;
    ctx.visit(LocationId::Glacier__Sea_Burial__Collapsing_Ceiling__Drown);

    let mut ctx = ContextWrapper::new(ctx);
    ctx.append_history(
        History::G(
            Item::Amashilama,
            LocationId::Glacier__Sea_Burial__Collapsing_Ceiling__Drown,
        ),
        20,
    );

    let buf = serde_pass(&ctx);
    let json = serde_json::to_string(&ctx).unwrap();

    let mut buf2 = Vec::new();
    ctx.get()
        .serialize(&mut Serializer::new(&mut buf2))
        .unwrap();
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
