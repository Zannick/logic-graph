pub mod heap;
pub mod db;

pub use heap::LimitedHeap;

#[macro_export]
macro_rules! expect_no_route {
    ($world:expr, $ctx:expr, $T:ty, $start:expr, $end:expr) => {{
        $ctx.set_position($start, $world);

        let spot_map = $crate::access::accessible_spots(
            $world,
            $crate::context::ContextWrapper::new($ctx),
            u32::MAX,
            true,
        );
        if let Some(ctx) = &spot_map.get(&$end) {
            panic!(
                "Found unexpected route from {} to {}:\n{}\n",
                $start,
                $end,
                $crate::context::history_str::<$T, _>(ctx.recent_history().iter().copied())
            );
        }
    }};
}

#[macro_export]
macro_rules! expect_any_route {
    ($world:expr, $ctx:expr, $start:expr, $end:expr) => {{
        $ctx.set_position($start, $world);

        let spot_map = $crate::access::accessible_spots(
            $world,
            $crate::context::ContextWrapper::new($ctx),
            u32::MAX,
            true,
        );
        let mut seen_spots: Vec<String> = spot_map
            .keys()
            .into_iter()
            .map(|s| format!("{}", s))
            .collect();
        seen_spots.sort();
        assert!(
            spot_map.contains_key(&$end),
            "Found no route from {} to {}:\n{}\nReached: {}\n",
            $start,
            $end,
            $crate::access::find_unused_links($world, &spot_map),
            seen_spots.join(", "),
        );
    }};
}

#[macro_export]
macro_rules! expect_this_route {
    ($world:expr, $ctx:expr, $start:expr, $spot_vec:expr) => {{
        $ctx.set_position($start, $world);
        let mut errors = Vec::new();

        'spots: for next_spot in $spot_vec {
            errors.clear();
            let movement_state = $ctx.get_movement_state($world);
            if $world.get_area_spots($ctx.position()).contains(&next_spot) {
                if $ctx.local_travel_time(movement_state, next_spot) > 0 {
                    $ctx.set_position(next_spot, $world);
                    continue;
                } else if $ctx.position() == next_spot {
                    errors.push(format!(
                        "attempting to move to current position: {}",
                        next_spot
                    ));
                } else if $world.are_spots_connected($ctx.position(), next_spot) {
                    errors.push(String::from("local travel not available"));
                } else {
                    errors.push(format!(
                        "Spot isn't connected from current position: {}",
                        next_spot
                    ));
                }
            }
            for exit in $world.get_spot_exits($ctx.position()) {
                if exit.dest() == next_spot {
                    if exit.can_access(&$ctx, $world) {
                        $ctx.set_position(next_spot, $world);
                        continue 'spots;
                    } else {
                        errors.push(format!(
                            "cannot use exit {}:\n{}",
                            exit.id(),
                            exit.explain(&$ctx, $world)
                        ));
                    }
                }
            }
            for warp in $world.get_warps() {
                if warp.dest(&$ctx, $world) == next_spot {
                    if warp.can_access(&$ctx, $world) {
                        warp.prewarp(&mut $ctx, $world);
                        $ctx.set_position(warp.dest(&$ctx, $world), $world);
                        $ctx.spend(&warp.price(&$ctx, $world));
                        warp.postwarp(&mut $ctx, $world);
                        if warp.should_reload() {
                            $ctx.reload_game($world);
                        }
                        continue 'spots;
                    } else {
                        errors.push(format!(
                            "cannot use warp {}:\n{}",
                            warp.id(),
                            warp.explain(&$ctx, $world)
                        ));
                    }
                }
            }
            if errors.is_empty() {
                errors.push(format!(
                    "No exits or warps to {}. Current area spots: {:?}",
                    next_spot,
                    $world.get_area_spots($ctx.position())
                ));
            }
            panic!(
                "Path breaks at {}: cannot get to {}:\n{}\n",
                $ctx.position(),
                next_spot,
                errors.join("\n")
            );
        }
    }};
}

#[macro_export]
macro_rules! expect_obtainable {
    ($world:expr, $ctx:expr, $start:expr, $item:expr) => {{
        $ctx.set_position($start, $world);

        let locations: Vec<_> = $world
            .get_all_locations()
            .iter()
            .filter_map(|loc| {
                if loc.item() == $item && $ctx.todo(loc) {
                    Some(loc.id())
                } else {
                    None
                }
            })
            .collect();
        assert!(
            !locations.is_empty(),
            "No unvisited locations have item {}",
            $item
        );
        let spot_map = $crate::access::accessible_spots(
            $world,
            $crate::context::ContextWrapper::new($ctx),
            u32::MAX,
            true,
        );
        let mut errors = Vec::new();
        let mut done = false;
        for loc in locations {
            let spot = $world.get_location_spot(loc);
            if let Some(ctx) = &spot_map.get(&spot) {
                if $world.get_location(loc).can_access(ctx.get(), $world) {
                    done = true;
                    break;
                }
                errors.push(format!(
                    "Unable to access location {}:\n{}",
                    loc,
                    $world.get_location(loc).explain(ctx.get(), $world)
                ));
            } else {
                errors.push(format!("Unable to reach spot {}", spot));
            }
        }
        if !done {
            let mut keys: Vec<_> = spot_map.keys().map(|k| format!("{}", k)).collect();
            keys.sort_unstable();
            panic!(
                "Unable to reach any unvisited location with {}:\n{}\nSpots reached:\n{}\n",
                $item,
                errors.join("\n"),
                keys.join("\n")
            );
        }
    }};
}

#[macro_export]
macro_rules! expect_not_obtainable {
    ($world:expr, $ctx:expr, $T:ty, $start:expr, $item:expr) => {{
        $ctx.set_position($start, $world);

        let locations: Vec<_> = $world
            .get_all_locations()
            .iter()
            .filter_map(|loc| {
                if loc.item() == $item && $ctx.todo(loc) {
                    Some(loc.id())
                } else {
                    None
                }
            })
            .collect();
        assert!(
            !locations.is_empty(),
            "Test not meaningful: No unvisited locations have item {}",
            $item
        );
        let spot_map = $crate::access::accessible_spots(
            $world,
            $crate::context::ContextWrapper::new($ctx),
            u32::MAX,
            true,
        );
        for loc in locations {
            let spot = $world.get_location_spot(loc);
            if let Some(ctx) = &spot_map.get(&spot) {
                assert!(
                    !$world.get_location(loc).can_access(ctx.get(), $world),
                    "Able to access location {}:\n{}\n{}\n",
                    loc,
                    $world.get_location(loc).explain(ctx.get(), $world),
                    $crate::context::history_str::<$T, _>(ctx.recent_history().iter().copied())
                );
            }
        }
    }};
}

#[macro_export]
macro_rules! expect_accessible {
    ($world:expr, $ctx:expr, $start:expr, $loc_id:expr) => {{
        $ctx.set_position($start, $world);

        let spot_map = $crate::access::accessible_spots(
            $world,
            $crate::context::ContextWrapper::new($ctx),
            u32::MAX,
            true,
        );
        let spot = $world.get_location_spot($loc_id);
        if let Some(ctx) = &spot_map.get(&spot) {
            assert!(
                $world.get_location($loc_id).can_access(ctx.get(), $world),
                "Expected location {} to be accessible, but was not:\n{}",
                $loc_id,
                $world.get_location($loc_id).explain(ctx.get(), $world)
            );
        } else {
            panic!("Unable to reach spot containing location: {}", spot);
        }
    }};
}

#[macro_export]
macro_rules! expect_inaccessible {
    ($world:expr, $ctx:expr, $T:ty, $start:expr, $loc_id:expr) => {{
        $ctx.set_position($start, $world);

        let spot_map = $crate::access::accessible_spots(
            $world,
            $crate::context::ContextWrapper::new($ctx),
            u32::MAX,
            true,
        );
        let spot = $world.get_location_spot($loc_id);
        if let Some(ctx) = &spot_map.get(&spot) {
            assert!(
                !$world.get_location($loc_id).can_access(ctx.get(), $world),
                "Expected location {} to be inaccessible, but was accessible:\n{}\n{}",
                $loc_id,
                $world.get_location($loc_id).explain(ctx.get(), $world),
                $crate::context::history_str::<$T, _>(ctx.recent_history().iter().copied())
            );
        }
    }};
}

#[macro_export]
macro_rules! expect_action_accessible {
    ($world:expr, $ctx:expr, $start:expr, $act_id:expr) => {{
        $ctx.set_position($start, $world);

        if $world.is_global_action($act_id) {
            assert!(
                $world.get_action($act_id).can_access(&$ctx, $world),
                "Expected global action {} to be accessible, but was not:\n{}",
                $act_id,
                $world.get_action($act_id).explain(&$ctx, $world)
            );
        }

        let spot_map = $crate::access::accessible_spots(
            $world,
            $crate::context::ContextWrapper::new($ctx),
            u32::MAX,
            true,
        );
        let spot = $world.get_action_spot($act_id);
        if let Some(ctx) = &spot_map.get(&spot) {
            assert!(
                $world.get_action($act_id).can_access(ctx.get(), $world),
                "Expected action {} to be accessible, but was not:\n{}",
                $act_id,
                $world.get_action($act_id).explain(ctx.get(), $world)
            );
        } else {
            panic!("Unable to reach spot containing action: {}", spot);
        }
    }};
}

#[macro_export]
macro_rules! expect_action_inaccessible {
    ($world:expr, $ctx:expr, $start:expr, $act_id:expr) => {{
        $ctx.set_position($start, $world);

        if $world.is_global_action($act_id) {
            assert!(
                !$world.get_action($act_id).can_access(&$ctx, $world),
                "Expected global action {} to be inaccessible, but was accessible:\n{}",
                $act_id,
                $world.get_action($act_id).explain(&$ctx, $world)
            );
        }

        let spot_map = $crate::access::accessible_spots(
            $world,
            $crate::context::ContextWrapper::new($ctx),
            u32::MAX,
            true,
        );
        let spot = $world.get_action_spot($act_id);
        if let Some(ctx) = &spot_map.get(&spot) {
            assert!(
                !$world.get_action($act_id).can_access(ctx.get(), $world),
                "Expected action {} to be inaccessible, but was accessible:\n{}",
                $act_id,
                $world.get_action($act_id).explain(ctx.get(), $world)
            );
        }
    }};
}

// TODO: should eventually be using greedy search instead?
#[macro_export]
macro_rules! expect_eventually_gets {
    ($world:expr, $ctx:expr, $start:expr, $item:expr) => {{
        $ctx.set_position($start, $world);

        let item_locs: Vec<_> = $world
            .get_all_locations()
            .iter()
            .filter_map(|loc| {
                if loc.item() == $item {
                    Some(loc.id())
                } else {
                    None
                }
            })
            .collect();

        assert!(item_locs.len() > 0, "Found no locations with {}", $item);

        let mut heap = $crate::testlib::LimitedHeap::new();
        heap.push($crate::context::ContextWrapper::new($ctx));
        let mut count = 1000;
        let mut done = false;
        while let Some(ctx) = heap.pop() {
            if item_locs.iter().any(|loc_id| ctx.get().visited(*loc_id)) {
                done = true;
                break;
            }
            if count == 0 {
                panic!("Did not find {} in the iteration limit", $item);
            }
            heap.extend($crate::search::classic_step($world, ctx, u32::MAX));
            count -= 1;
        }
        if !done {
            panic!("Dead-ended without finding {}", $item);
        }
    }};
}

#[macro_export]
macro_rules! expect_eventually_reaches {
    ($world:expr, $ctx:expr, $start:expr, $spot:expr) => {{
        $ctx.set_position($start, $world);

        let mut heap = $crate::testlib::LimitedHeap::new();
        heap.push($crate::context::ContextWrapper::new($ctx));
        let mut count = 1000;
        let mut done = false;
        while let Some(ctx) = heap.pop() {
            if ctx.get().position() == $spot {
                done = true;
                break;
            }
            if count == 0 {
                panic!("Did not reach {} in the iteration limit", $spot);
            }
            heap.extend($crate::search::classic_step($world, ctx, u32::MAX));
            count -= 1;
        }
        if !done {
            panic!("Dead-ended without reaching {}", $spot);
        }
    }};
}

#[macro_export]
macro_rules! expect_eventually_accesses {
    ($world:expr, $ctx:expr, $start:expr, $loc_id:expr) => {{
        $ctx.set_position($start, $world);

        if !$ctx.visited($loc_id) {
            assert!(
                !$ctx.visited($loc_id),
                "Expected {} to be unvisited",
                $loc_id
            );

            let mut heap = $crate::testlib::LimitedHeap::new();
            heap.push($crate::context::ContextWrapper::new($ctx));
            let mut count = 1000;
            while let Some(ctx) = heap.pop() {
                if !ctx.get().visited($loc_id) {
                    if count == 0 {
                        panic!("Did not visit {} in the iteration limit", $loc_id);
                    }
                    heap.extend($crate::search::classic_step($world, ctx, u32::MAX));
                    count -= 1;
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! expect_eventually_activates {
    ($world:expr, $ctx:expr, $start:expr, $act_id:expr) => {{
        $ctx.set_position($start, $world);

        let mut heap = $crate::testlib::LimitedHeap::new();
        heap.push($crate::context::ContextWrapper::new($ctx));
        let mut count = 1000;
        let mut done = false;
        while let Some(ctx) = heap.pop() {
            if let Some($crate::context::History::A(a)) = ctx.recent_history().last() {
                if *a == $act_id {
                    done = true;
                    break;
                }
            }
            if count == 0 {
                panic!("Did not activate {} in the iteration limit", $act_id);
            }
            heap.extend($crate::search::classic_step($world, ctx, u32::MAX));
            count -= 1;
        }
        if !done {
            panic!("Dead-ended without activating {}", $act_id);
        }
    }};
}

#[macro_export]
macro_rules! expect_eventually_requires {
    ($world:expr, $ctx:expr, $T:ty, $start:expr, $test_req:expr, $verify_req:expr, $limit:expr, $desc:expr, $cont:expr) => {{
        $ctx.set_position($start, $world);

        let mut heap = $crate::testlib::LimitedHeap::new();
        heap.push($crate::context::ContextWrapper::new($ctx));
        let mut count = $limit;
        let mut success = false;
        let mut done = false;
        while let Some(ctx) = heap.pop() {
            if ($test_req)(ctx.get()) {
                let result = ($verify_req)(ctx.get());
                assert!(
                    result.is_ok(),
                    "Unexpectedly able to {} without requirements:\n{}\n{}\n",
                    $desc,
                    result.unwrap_err(),
                    $crate::context::history_str::<$T, _>(ctx.recent_history().iter().copied()),
                );
                success = true;
            }
            if ($cont)(ctx) {
                if count == 0 {
                    assert!(
                        success,
                        "Did not {} in the iteration limit of {}",
                        $desc, $limit
                    );
                    done = true;
                }
                heap.extend($crate::search::classic_step($world, ctx, u32::MAX));
                count -= 1;
            }
        }
        if !done {
            assert!(success, "Dead-ended: did not {}", $desc);
        }
    }};
}

#[macro_export]
macro_rules! expect_eventually_requires_to_obtain {
    ($world:expr, $ctx:expr, $T:ty, $start:expr, $item:expr, $verify_req:expr, $limit:expr) => {{
        $ctx.set_position($start, $world);

        let mut heap = $crate::testlib::LimitedHeap::new();
        heap.push($crate::context::ContextWrapper::new($ctx));
        let mut count = $limit;
        let mut success = false;
        let mut done = false;
        while let Some(ctx) = heap.pop() {
            if ctx.get().has($item) {
                let result = ($verify_req)(ctx.get());
                assert!(
                    result.is_ok(),
                    "Unexpectedly able to find {} without requirements:\n{}\n{}\n",
                    $item,
                    result.unwrap_err(),
                    $crate::context::history_str::<$T, _>(ctx.recent_history().iter().copied()),
                );
                success = true;
            }
            if count == 0 {
                assert!(
                    success,
                    "Did not find {} in the iteration limit of {}",
                    $item, $limit
                );
                done = true;
                break;
            }
            heap.extend($crate::search::classic_step($world, ctx, u32::MAX));
            count -= 1;
        }
        if !done {
            assert!(success, "Dead-ended: did not find {}", $item);
        }
    }};
}

#[macro_export]
macro_rules! expect_eventually_requires_to_reach {
    ($world:expr, $ctx:expr, $T:ty, $start:expr, $spot:expr, $verify_req:expr, $limit:expr) => {{
        $ctx.set_position($start, $world);

        let mut heap = $crate::testlib::LimitedHeap::new();
        heap.push($crate::context::ContextWrapper::new($ctx));
        let mut count = $limit;
        let mut success = false;
        let mut done = false;
        while let Some(ctx) = heap.pop() {
            if ctx.get().position() == $spot {
                let result = ($verify_req)(ctx.get());
                assert!(
                    result.is_ok(),
                    "Unexpectedly able to reach {} without requirements:\n{}\n{}\n",
                    $spot,
                    result.unwrap_err(),
                    $crate::context::history_str::<$T, _>(ctx.recent_history().iter().copied()),
                );
                success = true;
            }
            if count == 0 {
                assert!(
                    success,
                    "Did not reach {} in the iteration limit of {}",
                    $spot, $limit
                );
                done = true;
                break;
            }
            heap.extend($crate::search::classic_step($world, ctx, u32::MAX));
            count -= 1;
        }
        if !done {
            assert!(success, "Dead-ended: did not reach {}", $spot);
        }
    }};
}
#[macro_export]
macro_rules! expect_eventually_requires_to_access {
    ($world:expr, $ctx:expr, $T:ty, $start:expr, $loc_id:expr, $verify_req:expr, $limit:expr) => {{
        $ctx.set_position($start, $world);

        let mut heap = $crate::testlib::LimitedHeap::new();
        heap.push($crate::context::ContextWrapper::new($ctx));
        let mut count = $limit;
        let mut success = false;
        let mut done = false;
        while let Some(ctx) = heap.pop() {
            if ctx.get().visited($loc_id) {
                let result = ($verify_req)(ctx.get());
                assert!(
                    result.is_ok(),
                    "Unexpectedly able to visit {} without requirements:\n{}\n{}\n",
                    $loc_id,
                    result.unwrap_err(),
                    $crate::context::history_str::<$T, _>(ctx.recent_history().iter().copied()),
                );
                success = true;
            } else {
                if count == 0 {
                    assert!(
                        success,
                        "Did not visit {} in the iteration limit of {}",
                        $loc_id, $limit
                    );
                    done = true;
                    break;
                }
                heap.extend($crate::search::classic_step($world, ctx, u32::MAX));
                count -= 1;
            }
        }
        if !done {
            assert!(success, "Dead-ended: did not visit {}", $loc_id);
        }
    }};
}
#[macro_export]
macro_rules! expect_eventually_requires_to_activate {
    ($world:expr, $ctx:expr, $T:ty, $start:expr, $act_id:expr, $verify_req:expr, $limit:expr) => {{
        $ctx.set_position($start, $world);

        let mut heap = $crate::testlib::LimitedHeap::new();
        heap.push($crate::context::ContextWrapper::new($ctx));
        let mut count = $limit;
        let mut success = false;
        let mut done = false;
        while let Some(ctx) = heap.pop() {
            if let Some($crate::context::History::A(a)) = ctx.recent_history().last() {
                if *a == $act_id {
                    let result = ($verify_req)(ctx.get());
                    assert!(
                        result.is_ok(),
                        "Unexpectedly able to activate {} without requirements:\n{}\n{}\n",
                        $act_id,
                        result.unwrap_err(),
                        $crate::context::history_str::<$T, _>(ctx.recent_history().iter().copied()),
                    );
                    success = true;
                }
            }
            if count == 0 {
                assert!(
                    success,
                    "Did not activate {} in the iteration limit of {}",
                    $act_id, $limit
                );
                done = true;
                break;
            }
            heap.extend($crate::search::classic_step($world, ctx, u32::MAX));
            count -= 1;
        }
        if !done {
            assert!(success, "Dead-ended: did not activate {}", $act_id);
        }
    }};
}
