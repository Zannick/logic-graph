#![allow(unused)]

use analyzer::access::move_to;
use analyzer::context::{history_to_partial_route, ContextWrapper, Ctx, Wrapper};
use analyzer::db::RouteDb;
use analyzer::estimates::ContextScorer;
use analyzer::route::route_from_string;
use analyzer::testlib::db::all_keys_cf;
use analyzer::testlib::db::TestRouteDb;
use analyzer::world::World;
use base64::prelude::*;
use libsample::context::Context;
use libsample::graph;
use libsample::items::Item;

#[test]
fn test_route_db() {
    let db = TestRouteDb::<Context>::default();
    let world = graph::World::new();
    let mut startctx = Context::default();
    startctx.add_item(Item::Kokiri_Sword);
    startctx.add_item(Item::Buy_Deku_Shield);
    startctx.add_item(Item::Showed_Mido);
    let scorer = ContextScorer::shortest_paths(&*world, &startctx, 32_768);

    let ctx = move_to(
        &*world,
        ContextWrapper::new(startctx.clone()),
        graph::SpotId::KF__Kokiri_Village__Midos_Porch,
        scorer.get_algo(),
    )
    .unwrap();

    // Efficient route from start to Mido's Porch
    let route = history_to_partial_route(&startctx, &*world, ctx.recent_history().iter().copied());
    db.rdb.insert_route(
        &startctx,
        &*world,
        graph::SpotId::KF__Kokiri_Village__Midos_Porch,
        &route,
    );
    let midos = ctx.clone();

    let ctx = move_to(
        &*world,
        ctx,
        graph::SpotId::KF__Boulder_Maze__Reward,
        scorer.get_algo(),
    )
    .unwrap();
    // Inefficient route to Boulder Maze Reward by way of Mido's Porch
    let route = history_to_partial_route(&startctx, &*world, ctx.recent_history().iter().copied());
    db.rdb.insert_route(
        &startctx,
        &*world,
        graph::SpotId::KF__Boulder_Maze__Reward,
        &route,
    );

    let ctx2 = move_to(
        &*world,
        ContextWrapper::new(startctx.clone()),
        graph::SpotId::KF__Boulder_Maze__Reward,
        scorer.get_algo(),
    )
    .unwrap();

    assert!(ctx2.elapsed() < ctx.elapsed());

    let _r0 = db
        .rdb
        .best_known_route(&startctx, graph::SpotId::KF__Kokiri_Village__Midos_Porch)
        .unwrap()
        .expect("No route to Mido's found in db!");

    let r1 = db
        .rdb
        .best_known_route(&startctx, graph::SpotId::KF__Boulder_Maze__Reward)
        .unwrap()
        .expect("No route to Boulder Maze Reward found in db!");
    assert_eq!(&r1, &*route.route);

    assert!(
        db.rdb
            .best_known_route(midos.get(), graph::SpotId::KF__Boulder_Maze__Reward)
            .unwrap()
            .is_some(),
        "No route from Mido's to Boulder Maze Reward found in db!"
    );

    // Improved route
    let route = history_to_partial_route(&startctx, &*world, ctx2.recent_history().iter().copied());
    db.rdb.insert_route(
        &startctx,
        &*world,
        graph::SpotId::KF__Boulder_Maze__Reward,
        &route,
    );

    let r2 = db
        .rdb
        .best_known_route(&startctx, graph::SpotId::KF__Boulder_Maze__Reward)
        .unwrap()
        .expect("No route to Boulder Maze Reward after improvement?!");
    assert_eq!(&r2, &*route.route);

    let t1: u32 = r1.iter().map(|s| s.time).sum();
    let t2: u32 = r2.iter().map(|s| s.time).sum();
    assert!(t2 < t1);

    // Not an improvement
    let ctx3 = route_from_string(
        &*world,
        &startctx,
        r#"
    Move... to KF > Know-it-all House > Entry
    Move... to KF > Kokiri Village > Mido's Guardpost
    Move... to KF > Boulder Maze > Reward
    "#,
        scorer.get_algo(),
    )
    .unwrap();

    let route = history_to_partial_route(&startctx, &*world, ctx3.recent_history().iter().copied());
    db.rdb.insert_route(
        &startctx,
        &*world,
        graph::SpotId::KF__Boulder_Maze__Reward,
        &route,
    );

    let r3 = db
        .rdb
        .best_known_route(&startctx, graph::SpotId::KF__Boulder_Maze__Reward)
        .unwrap()
        .expect("No route to Boulder Maze Reward after non-improvement?!");
    assert_ne!(&r3, &*route.route);
}
