extern crate enum_map;
extern crate lru;
extern crate priority_queue;
extern crate rayon;
extern crate rmp_serde;
extern crate rustc_hash;
extern crate serde;
extern crate sort_by_derive;
extern crate yaml_rust;

pub mod access;
pub mod algo;
pub mod context;
pub mod db;
pub mod estimates;
pub mod greedy;
pub mod heap;
pub mod minimize;
pub mod settings;
pub mod solutions;
pub mod steiner;
pub mod world;

pub(crate) type CommonHasher = std::hash::BuildHasherDefault<rustc_hash::FxHasher>;
pub(crate) fn new_hashmap<T, U>() -> std::collections::HashMap<T, U, CommonHasher> {
    rustc_hash::FxHashMap::default()
}
pub(crate) fn new_hashset<T>() -> std::collections::HashSet<T, CommonHasher> {
    rustc_hash::FxHashSet::default()
}

pub mod testlib {
    #[macro_export]
    macro_rules! expect_no_route {
        ($world:expr, $ctx:expr, $start:expr, $end:expr) => {{
            $ctx.set_position($start);

            let spot_map = $crate::access::accessible_spots(
                $world,
                $crate::context::ContextWrapper::new($ctx),
                u32::MAX,
            );
            if let Some(ctx) = &spot_map[$end] {
                panic!(
                    "Found unexpected route from {} to {}:\n{}\n",
                    $start,
                    $end,
                    ctx.history_str()
                );
            }
        }};
    }

    #[macro_export]
    macro_rules! expect_any_route {
        ($world:expr, $ctx:expr, $start:expr, $end:expr) => {{
            $ctx.set_position($start);

            let spot_map = $crate::access::accessible_spots(
                $world,
                $crate::context::ContextWrapper::new($ctx),
                u32::MAX,
            );
            assert!(
                spot_map[$end] != None,
                "Found no route from {} to {}:\n{}\n",
                $start,
                $end,
                $crate::access::find_unused_links($world, &spot_map),
            );
        }};
    }

    #[macro_export]
    macro_rules! expect_this_route {
        ($world:expr, $ctx:expr, $start:expr, $spot_vec:expr) => {{
            $ctx.set_position($start);
            let mut errors = Vec::new();

            'spots: for next_spot in $spot_vec {
                errors.clear();
                let movement_state = $ctx.get_movement_state();
                if $world.get_area_spots($ctx.position()).contains(&next_spot) {
                    if $ctx.local_travel_time(movement_state, next_spot) > 0 {
                        $ctx.set_position(next_spot);
                        continue;
                    } else if $world.are_spots_connected($ctx.position(), next_spot) {
                        errors.push(String::from("local travel not available"));
                    }
                }
                for exit in $world.get_spot_exits($ctx.position()) {
                    if exit.dest() == next_spot {
                        if exit.can_access(&$ctx) {
                            $ctx.set_position(next_spot);
                            continue 'spots;
                        } else {
                            errors.push(format!("cannot use exit {}", exit.id()));
                        }
                    }
                }
                for warp in $world.get_warps() {
                    if warp.dest(&$ctx) == next_spot {
                        if warp.can_access(&$ctx) {
                            $ctx.set_position(next_spot);
                            continue 'spots;
                        } else {
                            errors.push(format!("cannot use warp {}", warp.id()));
                        }
                    }
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
            $ctx.set_position($start);

            let locations: Vec<_> = $world
                .get_all_locations()
                .iter()
                .filter_map(|loc| {
                    if loc.item() == $item && $ctx.todo(loc.id()) {
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
            );
            let mut errors = Vec::new();
            for loc in locations {
                let spot = $world.get_location_spot(loc);
                if let Some(ctx) = &spot_map[spot] {
                    if $world.get_location(loc).can_access(ctx.get()) {
                        return;
                    }
                    errors.push(format!("Unable to access location {}", loc));
                } else {
                    errors.push(format!("Unable to reach spot {}", spot));
                }
            }
            panic!(
                "Unable to reach any unvisited location with {}:\n{}\n",
                $item,
                errors.join("\n")
            );
        }};
    }

    #[macro_export]
    macro_rules! expect_not_obtainable {
        ($world:expr, $ctx:expr, $start:expr, $item:expr) => {{
            $ctx.set_position($start);

            let locations: Vec<_> = $world
                .get_all_locations()
                .iter()
                .filter_map(|loc| {
                    if loc.item() == $item && $ctx.todo(loc.id()) {
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
            );
            for loc in locations {
                let spot = $world.get_location_spot(loc);
                if let Some(ctx) = &spot_map[spot] {
                    assert!(
                        !$world.get_location(loc).can_access(ctx.get()),
                        "Able to access location {}:\n{}\n",
                        loc,
                        ctx.history_str()
                    );
                }
            }
        }};
    }

    #[macro_export]
    macro_rules! expect_accessible {
        ($world:expr, $ctx:expr, $start:expr, $loc_id:expr) => {{
            $ctx.set_position($start);

            let spot_map = $crate::access::accessible_spots(
                $world,
                $crate::context::ContextWrapper::new($ctx),
                u32::MAX,
            );
            let spot = $world.get_location_spot($loc_id);
            if let Some(ctx) = &spot_map[spot] {
                assert!(
                    $world.get_location($loc_id).can_access(ctx.get()),
                    "Expected location {} to be accessible",
                    $loc_id
                );
            } else {
                panic!("Unable to reach spot containing location: {}", spot);
            }
        }};
    }

    #[macro_export]
    macro_rules! expect_inaccessible {
        ($world:expr, $ctx:expr, $start:expr, $loc_id:expr) => {{
            $ctx.set_position($start);

            let spot_map = $crate::access::accessible_spots(
                $world,
                $crate::context::ContextWrapper::new($ctx),
                u32::MAX,
            );
            let spot = $world.get_location_spot($loc_id);
            if let Some(ctx) = &spot_map[spot] {
                assert!(
                    !$world.get_location($loc_id).can_access(ctx.get()),
                    "Expected location {} to be inaccessible:\n{}",
                    $loc_id,
                    ctx.history_str()
                );
            }
        }};
    }

    #[macro_export]
    macro_rules! expect_action_accessible {
        ($world:expr, $ctx:expr, $start:expr, $act_id:expr) => {{
            $ctx.set_position($start);

            if $world.is_global_action($act_id) {
                assert!(
                    $world.get_action($act_id).can_access(&$ctx),
                    "Expected global action {} to be accessible",
                    $act_id
                );
            }

            let spot_map = $crate::access::accessible_spots(
                $world,
                $crate::context::ContextWrapper::new($ctx),
                u32::MAX,
            );
            let spot = $world.get_action_spot($act_id);
            if let Some(ctx) = &spot_map[spot] {
                assert!(
                    $world.get_action($act_id).can_access(ctx.get()),
                    "Expected action {} to be accessible",
                    $act_id
                );
            } else {
                panic!("Unable to reach spot containing action: {}", spot);
            }
        }};
    }

    #[macro_export]
    macro_rules! expect_action_inaccessible {
        ($world:expr, $ctx:expr, $start:expr, $act_id:expr) => {{
            $ctx.set_position($start);

            if $world.is_global_action($act_id) {
                assert!(
                    !$world.get_action($act_id).can_access(&$ctx),
                    "Expected global action {} to be inaccessible",
                    $act_id
                );
            }

            let spot_map = $crate::access::accessible_spots(
                $world,
                $crate::context::ContextWrapper::new($ctx),
                u32::MAX,
            );
            let spot = $world.get_action_spot($act_id);
            if let Some(ctx) = &spot_map[spot] {
                assert!(
                    !$world.get_action($act_id).can_access(ctx.get()),
                    "Expected action {} to be inaccessible",
                    $act_id
                );
            }
        }};
    }

    // TODO: should eventually be using greedy search instead?
    #[macro_export]
    macro_rules! expect_eventually_gets {
        ($world:expr, $ctx:expr, $start:expr, $item:expr) => {{
            $ctx.set_position($start);

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

            let mut heap = $crate::heap::LimitedHeap::new();
            heap.push($crate::context::ContextWrapper::new($ctx));
            let mut count = 1000;
            while let Some(ctx) = heap.pop() {
                if item_locs.iter().any(|loc_id| ctx.get().visited(*loc_id)) {
                    return;
                }
                if count == 0 {
                    panic!("Did not find {} in the iteration limit", $item);
                }
                heap.extend($crate::algo::classic_step($world, ctx, u32::MAX));
                count -= 1;
            }
            panic!("Dead-ended without finding {}", $item);
        }};
    }

    #[macro_export]
    macro_rules! expect_eventually_reaches {
        ($world:expr, $ctx:expr, $start:expr, $spot:expr) => {{
            $ctx.set_position($start);

            let mut heap = $crate::heap::LimitedHeap::new();
            heap.push($crate::context::ContextWrapper::new($ctx));
            let mut count = 1000;
            while let Some(ctx) = heap.pop() {
                if ctx.get().position() == $spot {
                    return;
                }
                if count == 0 {
                    panic!("Did not reach {} in the iteration limit", $spot);
                }
                heap.extend($crate::algo::classic_step($world, ctx, u32::MAX));
                count -= 1;
            }
            panic!("Dead-ended without reaching {}", $spot);
        }};
    }

    #[macro_export]
    macro_rules! expect_eventually_accesses {
        ($world:expr, $ctx:expr, $start:expr, $loc_id:expr) => {{
            $ctx.set_position($start);

            if $ctx.visited($loc_id) {
                return;
            }
            assert!(
                $ctx.todo($loc_id),
                "Expected {} to be unvisited and unskipped",
                $loc_id
            );

            let mut heap = $crate::heap::LimitedHeap::new();
            heap.push($crate::context::ContextWrapper::new($ctx));
            let mut count = 1000;
            while let Some(ctx) = heap.pop() {
                if ctx.get().todo($loc_id) {
                    if count == 0 {
                        panic!("Did not visit {} in the iteration limit", $loc_id);
                    }
                    heap.extend($crate::algo::classic_step($world, ctx, u32::MAX));
                    count -= 1;
                } else if ctx.get().visited($loc_id) {
                    return;
                }
                // if we skipped the location, don't bother with expanding that line further
            }
            panic!("Dead-ended without visiting {}", $loc_id);
        }};
    }

    #[macro_export]
    macro_rules! expect_eventually_activates {
        ($world:expr, $ctx:expr, $start:expr, $act_id:expr) => {{
            $ctx.set_position($start);

            let mut heap = $crate::heap::LimitedHeap::new();
            heap.push($crate::context::ContextWrapper::new($ctx));
            let mut count = 1000;
            while let Some(ctx) = heap.pop() {
                if let Some($crate::context::History::Activate(a)) = ctx.last_step() {
                    if a == $act_id {
                        return;
                    }
                }
                if count == 0 {
                    panic!("Did not activate {} in the iteration limit", $act_id);
                }
                heap.extend($crate::algo::classic_step($world, ctx, u32::MAX));
                count -= 1;
            }
            panic!("Dead-ended without activating {}", $act_id);
        }};
    }

    #[macro_export]
    macro_rules! expect_eventually_requires {
        ($world:expr, $ctx:expr, $start:expr, $test_req:expr, $verify_req:expr, $limit:expr, $desc:expr, $cont:expr) => {{
            $ctx.set_position($start);

            let mut heap = $crate::heap::LimitedHeap::new();
            heap.push($crate::context::ContextWrapper::new($ctx));
            let mut count = $limit;
            let mut success = false;
            while let Some(ctx) = heap.pop() {
                if ($test_req)(ctx.get()) {
                    let result = ($verify_req)(ctx.get());
                    assert!(
                        result.is_ok(),
                        "Unexpectedly able to {} without requirements:\n{}\n{}\n",
                        $desc,
                        result.unwrap_err(),
                        ctx.history_str(),
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
                        return;
                    }
                    heap.extend($crate::algo::classic_step($world, ctx, u32::MAX));
                    count -= 1;
                }
            }
            assert!(success, "Dead-ended: did not {}", $desc);
        }};
    }

    #[macro_export]
    macro_rules! expect_eventually_requires_to_obtain {
        ($world:expr, $ctx:expr, $start:expr, $item:expr, $verify_req:expr, $limit:expr) => {{
            $ctx.set_position($start);

            let mut heap = $crate::heap::LimitedHeap::new();
            heap.push($crate::context::ContextWrapper::new($ctx));
            let mut count = $limit;
            let mut success = false;
            while let Some(ctx) = heap.pop() {
                if ctx.get().has($item) {
                    let result = ($verify_req)(ctx.get());
                    assert!(
                        result.is_ok(),
                        "Unexpectedly able to find {} without requirements:\n{}\n{}\n",
                        $item,
                        result.unwrap_err(),
                        ctx.history_str(),
                    );
                    success = true;
                }
                if count == 0 {
                    assert!(
                        success,
                        "Did not find {} in the iteration limit of {}",
                        $item, $limit
                    );
                    return;
                }
                heap.extend($crate::algo::classic_step($world, ctx, u32::MAX));
                count -= 1;
            }
            assert!(success, "Dead-ended: did not find {}", $item);
        }};
    }

    #[macro_export]
    macro_rules! expect_eventually_requires_to_reach {
        ($world:expr, $ctx:expr, $start:expr, $spot:expr, $verify_req:expr, $limit:expr) => {{
            $ctx.set_position($start);

            let mut heap = $crate::heap::LimitedHeap::new();
            heap.push($crate::context::ContextWrapper::new($ctx));
            let mut count = $limit;
            let mut success = false;
            while let Some(ctx) = heap.pop() {
                if ctx.get().position() == $spot {
                    let result = ($verify_req)(ctx.get());
                    assert!(
                        result.is_ok(),
                        "Unexpectedly able to reach {} without requirements:\n{}\n{}\n",
                        $spot,
                        result.unwrap_err(),
                        ctx.history_str(),
                    );
                    success = true;
                }
                if count == 0 {
                    assert!(
                        success,
                        "Did not reach {} in the iteration limit of {}",
                        $spot, $limit
                    );
                    return;
                }
                heap.extend($crate::algo::classic_step($world, ctx, u32::MAX));
                count -= 1;
            }
            assert!(success, "Dead-ended: did not reach {}", $spot);
        }};
    }
    #[macro_export]
    macro_rules! expect_eventually_requires_to_access {
        ($world:expr, $ctx:expr, $start:expr, $loc_id:expr, $verify_req:expr, $limit:expr) => {{
            $ctx.set_position($start);

            let mut heap = $crate::heap::LimitedHeap::new();
            heap.push($crate::context::ContextWrapper::new($ctx));
            let mut count = $limit;
            let mut success = false;
            while let Some(ctx) = heap.pop() {
                if ctx.get().visited($loc_id) {
                    let result = ($verify_req)(ctx.get());
                    assert!(
                        result.is_ok(),
                        "Unexpectedly able to visit {} without requirements:\n{}\n{}\n",
                        $loc_id,
                        result.unwrap_err(),
                        ctx.history_str(),
                    );
                    success = true;
                }
                if ctx.get().todo($loc_id) {
                    if count == 0 {
                        assert!(
                            success,
                            "Did not visit {} in the iteration limit of {}",
                            $loc_id, $limit
                        );
                        return;
                    }
                    heap.extend($crate::algo::classic_step($world, ctx, u32::MAX));
                    count -= 1;
                }
            }
            assert!(success, "Dead-ended: did not visit {}", $loc_id);
        }};
    }
    #[macro_export]
    macro_rules! expect_eventually_requires_to_activate {
        ($world:expr, $ctx:expr, $start:expr, $act_id:expr, $verify_req:expr, $limit:expr) => {{
            $ctx.set_position($start);

            let mut heap = $crate::heap::LimitedHeap::new();
            heap.push($crate::context::ContextWrapper::new($ctx));
            let mut count = $limit;
            let mut success = false;
            while let Some(ctx) = heap.pop() {
                if let Some($crate::context::History::Activate(a)) = ctx.last_step() {
                    if a == $act_id {
                        let result = ($verify_req)(ctx.get());
                        assert!(
                            result.is_ok(),
                            "Unexpectedly able to activate {} without requirements:\n{}\n{}\n",
                            $act_id,
                            result.unwrap_err(),
                            ctx.history_str(),
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
                    return;
                }
                heap.extend($crate::algo::classic_step($world, ctx, u32::MAX));
                count -= 1;
            }
            assert!(success, "Dead-ended: did not activate {}", $act_id);
        }};
    }
}
