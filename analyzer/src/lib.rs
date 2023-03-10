extern crate sort_by_derive;
extern crate yaml_rust;

pub mod access;
pub mod algo;
pub mod context;
pub mod greedy;
pub mod heap;
pub mod minimize;
pub mod settings;
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
}
