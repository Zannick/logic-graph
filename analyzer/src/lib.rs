extern crate sort_by_derive;
extern crate yaml_rust;

pub mod access;
pub mod algo;
pub mod context;
pub mod greedy;
pub mod heap;
pub mod minimize;
pub mod settings;
pub mod solutions;
pub mod world;

pub mod testlib {
    #[macro_export]
    macro_rules! expect_no_route {
        ($world:expr, $ctx:expr, $start:expr, $end:expr) => {{
            $ctx.set_position($start);

            let spot_map = $crate::access::accessible_spots(
                $world,
                $crate::context::ContextWrapper::new($ctx),
            );
            if let Some(ctx) = spot_map.get(&$end) {
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
            );
            assert!(
                spot_map.contains_key(&$end),
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
                if $world.get_area_spots($ctx.position()).contains(&next_spot) {
                    if $ctx.local_travel_time(next_spot) > 0 {
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
            );
            let mut errors = Vec::new();
            for loc in locations {
                let spot = $world.get_location_spot(loc);
                if let Some(ctx) = spot_map.get(&spot) {
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
            );
            for loc in locations {
                let spot = $world.get_location_spot(loc);
                if let Some(ctx) = spot_map.get(&spot) {
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

    // TODO: should eventually be using greedy search instead?
    #[macro_export]
    macro_rules! expect_eventually {
        ($world:expr, $ctx:expr, $start:expr, $item:expr) => {{
            $ctx.set_position($start);

            let mut heap = $crate::heap::LimitedHeap::new();
            heap.push($crate::context::ContextWrapper::new($ctx));
            let mut count = 1000;
            while let Some(ctx) = heap.pop() {
                if ctx.get().has($item) {
                    return;
                }
                if count == 0 {
                    panic!("Did not find {} in the iteration limit", $item);
                }
                $crate::algo::search_step($world, ctx, &mut heap);
                count -= 1;
            }
            panic!("Dead-ended without finding {}", $item);
        }};
    }

    #[macro_export]
    macro_rules! expect_eventually_requires {
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
                    assert!(success, "Did not find {} in the iteration limit of {}", $item, $limit);
                    return;
                }
                $crate::algo::search_step($world, ctx, &mut heap);
                count -= 1;
            }
            assert!(success, "Dead-ended without finding {}", $item);
        }};
    }
}
