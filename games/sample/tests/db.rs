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

#[cfg(feature = "mysql")]
#[test]
fn test_mysql() {
    use std::time::Instant;

    use analyzer::{
        context::{history_to_full_data_series, History},
        models::{DBState, HistoryEntry, MySQLDB},
        new_hashmap,
        route::import_route_to_mysql,
        schema::db_states::dsl,
        scoring::{EstimatorWrapper, ScoreMetric, TimeSinceAndElapsed},
    };
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;
    use libsample::graph::SpotId;

    let world = graph::World::new();
    let mut startctx = Context::default();

    let metric = TimeSinceAndElapsed::new(&*world, &startctx);

    // slight inefficiency
    let route1 = r#"
    * Collect Kokiri_Sword from KF > Boulder Maze > Reward > Chest
      Move... to KF > Kokiri Village > Mido's Guardpost
    ! Do KF > Kokiri Village > Mido's Porch > Gather Rupees
    ! Do KF > Kokiri Village > Mido's Porch > Gather Rupees
    * Collect Buy_Deku_Shield from KF > Shop > Entry > Item 1
    * Collect Showed_Mido from KF > Kokiri Village > Mido's Guardpost > Show Mido
    "#;

    // improvement to state in the middle of the route
    let route2 = r#"
    * Collect Kokiri_Sword from KF > Boulder Maze > Reward > Chest
      Move... to KF > Kokiri Village > Mido's Porch
    "#;

    let mut ctx =
        route_from_string(&*world, &startctx, route1, metric.estimator().get_algo()).unwrap();
    let mut faster =
        route_from_string(&*world, &startctx, route2, metric.estimator().get_algo()).unwrap();

    let mut db = MySQLDB::with_test_connection(metric);

    let hist1 = ctx.remove_history().0;

    db.insert_one(&ContextWrapper::new(startctx.clone()), false, None, false)
        .unwrap();
    import_route_to_mysql(&*world, &startctx, &hist1, &mut db, None);
    let steps = hist1.len() + 1;
    assert_eq!(
        steps,
        dsl::db_states
            .count()
            .get_result::<i64>(db.connection())
            .unwrap() as usize
    );

    // the state in the better route is already in the db, but the time in the route is better
    assert!(db.exists(faster.get()).unwrap());
    assert!(faster.elapsed() < db.get_best_times(faster.get()).unwrap().elapsed);

    let old_record = db.get_record(faster.get()).unwrap();

    // now we add that route
    let hist2 = faster.remove_history().0;
    import_route_to_mysql(&*world, &startctx, &hist2, &mut db, None);
    // it updates some matching rows rather than insert new ones
    let steps2 = hist2.len() + 1;
    assert!(
        steps + steps2
            > dsl::db_states
                .count()
                .get_result::<i64>(db.connection())
                .unwrap() as usize
    );

    // the state in question has a better time
    assert!(faster.elapsed() == db.get_best_times(faster.get()).unwrap().elapsed);
    let new_record = db.get_record(faster.get()).unwrap();
    assert_ne!(old_record, new_record, "Record not updated at all");
    assert_eq!(
        old_record.state, new_record.state,
        "Records aren't for the same state"
    );
    assert_ne!(old_record.prev, new_record.prev);

    // remaining states do not
    let best_route = history_to_full_data_series(
        &startctx,
        &*world,
        hist2.iter().copied().chain(
            hist1
                .iter()
                .skip_while(|h| **h != History::L(SpotId::KF__Kokiri_Village__Midos_Porch))
                .skip(1)
                .copied(),
        ),
    );
    assert_eq!(
        best_route.last().unwrap().get(),
        ctx.get(),
        "Routes did not meet up"
    );
    // full_data_series is off by one due to start state at idx 0
    assert_eq!(faster.get(), best_route[hist2.len()].get());
    for i in hist2.len() + 1..best_route.len() {
        assert!(
            best_route[i].elapsed() < db.get_best_times(best_route[i].get()).unwrap().elapsed,
            "Step {} was not an improvement despite step {} improvement",
            i,
            hist2.len()
        );
    }

    let res = db.test_downstream(faster.get()).unwrap();
    // include the state that changed
    assert_eq!(
        res.len(),
        best_route.len() - hist2.len(),
        "Did not find the right number of downstream states"
    );
    for (ds_entry, wrapper) in res.iter().zip(best_route.iter().skip(hist2.len())) {
        assert_eq!(&ds_entry.state, wrapper.get(), "States not aligned.");
        assert_eq!(
            ds_entry.new_elapsed,
            wrapper.elapsed(),
            "State's new elapsed time not as expected in sql"
        );
        assert!(
            &ds_entry.state == faster.get() || ds_entry.old_elapsed > ds_entry.new_elapsed,
            "State meant to be improved is not improved: old={} new={}",
            ds_entry.old_elapsed,
            ds_entry.new_elapsed,
        );
    }

    // run the improvement. it should update exactly the later states
    assert_eq!(
        best_route.len() - hist2.len(),
        db.improve_downstream(faster.get()).unwrap()
    );
    for i in hist2.len()..best_route.len() {
        let exp = best_route[i].elapsed();
        let act = db.get_best_times(best_route[i].get()).unwrap().elapsed;
        assert_eq!(
            exp,
            act,
            "Step {} doesn't match after improvement (updated step {}): ctx={}, db={}",
            i,
            hist2.len(),
            exp,
            act,
        );
    }

    // get the full history of the final state
    let history = db.full_history(ctx.get()).unwrap();
    let mut state_id_map = new_hashmap();
    let mut next_id = 0;
    let mut missing = Vec::new();
    for h in &history {
        if !state_id_map.contains_key(&h.state) {
            state_id_map.insert(&h.state, next_id);
            next_id += 1;
        }
        if let Some(p) = &h.prev {
            if !state_id_map.contains_key(p) {
                state_id_map.insert(p, next_id);
                missing.push(next_id);
                next_id += 1;
            }
        }
    }

    assert!(
        missing.is_empty(),
        "Some prevs were not encountered or appeared later in the list: {:?}",
        missing
    );

    for (i, he) in history.iter().enumerate() {
        if i == 0 {
            assert_eq!(None, he.prev);
            assert_eq!(None, he.hist);
            assert_eq!(0, he.elapsed);
        } else {
            assert_eq!(
                he.prev.as_ref().unwrap(),
                &history[i - 1].state,
                "State {} ({:?}, {}) does not point to state {} ",
                i,
                he.hist,
                he.elapsed,
                i - 1,
            );
            assert_ne!(None, he.hist, "State {} has no step info", i);
        }
    }

    assert_eq!(best_route.len(), history.len());
    assert_eq!(
        best_route.last().unwrap().get(),
        &history.last().unwrap().state
    );
}
