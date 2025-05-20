#![allow(unused)]

use analyzer::access::{access_location_after_actions_heatmap, move_to};
use analyzer::context::{history_to_partial_route, ContextWrapper, Ctx, History, Wrapper};
use analyzer::db::RouteDb;
use analyzer::direct::DirectPathsMap;
use analyzer::estimates::ContextScorer;
use analyzer::matchertrie::IntegerObservation;
use analyzer::observer::Observer;
use analyzer::route::PartialRoute;
use analyzer::steiner::{build_simple_graph, EdgeId, NodeId, ShortestPaths, SteinerAlgo};
use analyzer::testlib::db::{all_keys_cf, TestRouteDb};
use analyzer::world::{Accessible, Action, Exit, Location, Warp, World as _};
use analyzer::*;
use base64::prelude::*;
use lazy_static::lazy_static;
use libaxiom_verge2::context::{enums, Context};
use libaxiom_verge2::graph::*;
use libaxiom_verge2::items::Item;
use libaxiom_verge2::observe::*;
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
        { ShortestPaths::from_graph(build_simple_graph(&**WORLD, &Context::default(), false)) };
    static ref DIRECT_PATHS: DirectPathsMap<
        World,
        Context,
        ObservationMatcher<PartialRoute<Context>, Option<PartialRoute<Context>>>,
    > = {
        DirectPathsMap::new(ContextScorer::shortest_paths_tree_free_edges(
            &**WORLD,
            &Context::default(),
        ))
    };
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
    assert!(exit1.can_access(&ctx, &**WORLD));
    let e1time = exit1.time(&ctx, &**WORLD);
    assert!(e1time > ltt);
    assert!(
        e1time > exit1.base_time(),
        "Expected a penalty to be applied"
    );
    let wrapper = ContextWrapper::new(ctx.clone());
    let result = move_to(
        &**WORLD,
        wrapper,
        SpotId::Amagi_Breach__East_Ruins__Northeast_Bubbles_Corner_Access,
        &*SPATHS,
    )
    .expect("didn't reach dest");
    assert!(
        result.elapsed() == ltt,
        "Expected move_to to reach dest in {} but got there in {}",
        ltt,
        result.elapsed()
    );

    ctx.set_position(
        SpotId::Amagi_Breach__East_Ruins__High_Rock_Lower_Ledge,
        &**WORLD,
    );
    let ltt2 = ctx.local_travel_time(
        movement_state,
        SpotId::Amagi_Breach__East_Ruins__Northeast_Bubbles_Center,
    );
    ctx.set_position(SpotId::Amagi_Breach__East_Ruins__Arch_West, &**WORLD);
    let exit3 =
        *WORLD.get_exit(ExitId::Amagi_Breach__East_Ruins__Arch_West__ex__High_Rock_Lower_Ledge_1);
    assert!(exit3.can_access(&ctx, &**WORLD));
    let e3time = exit3.time(&ctx, &**WORLD);

    let wrapper = ContextWrapper::new(ctx);
    let result = move_to(
        &**WORLD,
        wrapper,
        SpotId::Amagi_Breach__East_Ruins__Northeast_Bubbles_Corner_Access,
        &*SPATHS,
    )
    .expect("didn't reach dest");
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

#[test]
fn test_observations() {
    let mut ctx = Context::default();
    ctx.energy = 300;
    ctx.flasks = 1;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.add_item(Item::Fast_Travel);
    ctx.add_item(Item::Flask);
    ctx.add_item(Item::Remote_Drone);
    ctx.add_item(Item::Anuman);
    ctx.add_item(Item::Nanite_Mist);
    ctx.set_position(SpotId::Glacier__Revival__Save_Point, &**WORLD); // add map tile
    ctx.save = SpotId::Glacier__Revival__Save_Point;
    ctx.set_position(SpotId::Amagi__East_Lake__Arch_West, &**WORLD);
    ctx.visit(LocationId::Glacier__Sea_Burial__Collapsing_Ceiling__Drown);

    let mut full_obs = FullObservation::default();
    // Suppose we've already observed some things on this route.
    full_obs.observe_flask(IntegerObservation::Ge(1));
    full_obs.observe_mode();
    full_obs.apply_observations();
    let action = *WORLD.get_action(ActionId::Global__Become_Drone);
    assert!(action.observe_access(&ctx, &**WORLD, &mut full_obs));

    action.observe_effects(&mut ctx, &**WORLD, &mut full_obs);
    full_obs.apply_observations();
    let vec = full_obs.to_vec(&ctx);
    // Because we used the updated ctx, we should have observed mode is drone
    assert!(vec.contains(&OneObservation::Mode(enums::Mode::Drone)));

    ctx.set_position(SpotId::Menu__Kiengir_Map__Glacier_Revival, &**WORLD);
    let ft =
        History::E(ExitId::Menu__Kiengir_Map__Glacier_Revival__ex__Glacier__Revival__Save_Point_1);
    assert!(ctx.observe_replay(&**WORLD, ft, &mut full_obs));
    full_obs.apply_observations();
    let vec = full_obs.to_vec(&ctx);
    let strvec = format!("{:?}", vec);
    assert!(
        strvec.contains("MAP__GLACIER__REVIVAL__SAVE"),
        "map should be observed in: {}",
        strvec
    );

    ctx.set_position(SpotId::Glacier__Vertical_Room__Peak, &**WORLD);
    let cskip = History::V(
        Item::Flask,
        LocationId::Glacier__Vertical_Room__Peak__Flask_Fast_Travel,
        SpotId::Menu__Kiengir_Map__Glacier_Vertical_Room_Flask,
    );
    assert!(ctx.observe_replay(&**WORLD, cskip, &mut full_obs));
    full_obs.apply_observations();
    let vec = full_obs.to_vec(&ctx);
    let strvec = format!("{:?}", vec);
    // This is applied to ctx without any changes due to observe_replay.
    assert!(vec.contains(&OneObservation::FlaskGe(0, true)));
    assert!(
        strvec.contains("FAST_TRAVEL"),
        "fast travel should be observed in: {}",
        strvec
    );
}

#[test]
fn test_greedy_step() {
    let mut ctx = Context::default();
    ctx.energy = 300;
    ctx.flasks = 10;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.add_item(Item::Fast_Travel);
    ctx.add_item(Item::Flask);
    ctx.add_item(Item::Remote_Drone);
    ctx.add_item(Item::Anuman);
    ctx.add_item(Item::Nanite_Mist);
    ctx.add_item(Item::Exit_Breach);
    ctx.add_item(Item::Breach_Sight);
    ctx.add_item(Item::Breach_Attractor);
    ctx.add_item(Item::Shockwave);
    ctx.add_item(Item::Drone_Hover);
    ctx.add_item(Item::Slingshot_Hook);
    ctx.add_item(Item::Slingshot_Charge);
    ctx.add_item(Item::Nano_Lattice_2);
    ctx.add_item(Item::Glacier_Breach_Spidery_Connector_Gate);
    ctx.set_mode(enums::Mode::Drone);
    ctx.visit(LocationId::Glacier__Sea_Burial__Collapsing_Ceiling__Drown);
    // add map tiles
    ctx.set_position(SpotId::Glacier__Revival__Save_Point, &**WORLD);
    ctx.set_position(SpotId::Annuna__Upper_Save__Save_Point, &**WORLD);
    ctx.set_position(SpotId::Annuna__Center_Save__Save_Point, &**WORLD);
    ctx.set_position(SpotId::Ebih__Base_Camp__Save_Point, &**WORLD);
    ctx.set_position(SpotId::Irikar__Hub__Save_Point, &**WORLD);
    ctx.set_position(SpotId::Giguna__Ruins_Top__Save_Point, &**WORLD);
    ctx.set_position(SpotId::Giguna_Breach__Peak__Save_Point, &**WORLD);
    ctx.set_position(SpotId::Glacier_Breach__South_Save__Save_Point, &**WORLD);
    ctx.set_position(SpotId::Glacier_Breach__West_Save__Save_Point, &**WORLD);

    ctx.set_position(SpotId::Annuna__Egg_Room__Corner_Platform, &**WORLD);
    ctx.set_position(SpotId::Annuna__Egg_Room__East, &**WORLD);
    ctx.save = SpotId::Irikar__Hub__Save_Point;

    access_location_after_actions_heatmap(
        &**WORLD,
        ContextWrapper::new(ctx),
        LocationId::Amagi_Breach__Upper_Lake__Column__Health,
        1u32 << 20,
        4,
        65536 * 8,
        &SPATHS,
        &*DIRECT_PATHS,
    )
    .result()
    .unwrap();
}

#[test]
fn test_route_db() {
    let db = TestRouteDb::<Context>::default();
    let mut ctx = Context::default();
    ctx.energy = 300;
    ctx.flasks = 10;
    ctx.add_item(Item::Amashilama);
    ctx.add_item(Item::Ice_Axe);
    ctx.add_item(Item::Fast_Travel);
    ctx.add_item(Item::Flask);
    ctx.add_item(Item::Infect);
    ctx.add_item(Item::Remote_Drone);
    ctx.add_item(Item::Anuman);
    ctx.add_item(Item::Nanite_Mist);
    ctx.add_item(Item::Exit_Breach);
    ctx.add_item(Item::Breach_Sight);
    ctx.add_item(Item::Breach_Attractor);
    ctx.add_item(Item::Shockwave);
    ctx.add_item(Item::Drone_Hover);
    ctx.add_item(Item::Slingshot_Hook);
    ctx.add_item(Item::Slingshot_Charge);
    ctx.add_item(Item::Nano_Lattice_2);
    ctx.add_item(Item::Glacier_Breach_Spidery_Connector_Gate);
    ctx.add_item(Item::Hammond_Auth);
    ctx.set_mode(enums::Mode::Drone);
    ctx.visit(LocationId::Glacier__Sea_Burial__Collapsing_Ceiling__Drown);

    ctx.set_position(SpotId::Annuna__Filter_Teleporter__Egg, &**WORLD);
    let ctx2 = move_to(
        &**WORLD,
        ContextWrapper::new(ctx.clone()),
        SpotId::Annuna__Lamassu__Portal_Stand,
        &*SPATHS,
    )
    .unwrap();

    let route = history_to_partial_route(&ctx, &**WORLD, ctx2.recent_history().iter().copied());
    db.rdb.insert_route(
        &ctx,
        &**WORLD,
        SpotId::Annuna__Lamassu__Portal_Stand,
        &route,
    );

    let r1 = db
        .rdb
        .best_known_route(&ctx, SpotId::Annuna__Lamassu__Portal_Stand)
        .unwrap()
        .expect("No route to Annuna > Lamassu > Portal Stand found in db!");
    assert_eq!(&r1, &*route.route);
}
