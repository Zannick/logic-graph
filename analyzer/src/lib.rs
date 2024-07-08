extern crate bucket_queue;
extern crate clap;
extern crate enum_map;
extern crate libtest_mimic;
extern crate lru;
extern crate priority_queue;
extern crate rayon;
extern crate regex;
extern crate rmp_serde;
extern crate rustc_hash;
extern crate serde;
extern crate similar;
extern crate sort_by_derive;
extern crate yaml_rust;

mod a_star;
pub mod access;
pub mod bucket;
pub mod cli;
pub mod condense;
pub mod context;
pub mod db;
pub mod estimates;
pub mod greedy;
pub mod heap;
pub mod matchertrie;
pub mod minimize;
pub mod observer;
pub mod priority;
pub mod route;
pub mod search;
pub mod settings;
pub mod solutions;
pub mod steiner;
pub mod world;

// test-only
pub mod unittest;

pub type CommonHasher = std::hash::BuildHasherDefault<rustc_hash::FxHasher>;
pub fn new_hashmap<T, U>() -> std::collections::HashMap<T, U, CommonHasher> {
    rustc_hash::FxHashMap::default()
}
pub(crate) fn new_hashset<T>() -> std::collections::HashSet<T, CommonHasher> {
    rustc_hash::FxHashSet::default()
}
pub(crate) fn new_hashset_with<T>(el: T) -> std::collections::HashSet<T, CommonHasher>
where
    T: Eq + std::hash::Hash,
{
    let mut hs = new_hashset();
    hs.insert(el);
    hs
}

pub mod testlib {
    use crate::context::*;
    use crate::heap::HeapElement;
    use crate::CommonHasher;
    use lru::LruCache;
    use std::collections::BinaryHeap;
    use std::num::NonZeroUsize;

    /// A wrapper around a BinaryHeap of ContextWrapper<T> wherein:
    /// * items are sorted by a "score" combination of progress and elapsed time
    ///   (controlled by the ContextWrapper object)
    /// * a threshold of elapsed time can be set to make the heap ignore
    ///   items that have surpassed the elapsed time.
    pub struct LimitedHeap<T: Ctx> {
        max_time: u32,
        heap: BinaryHeap<HeapElement<T>>,
        states_seen: LruCache<T, u32, CommonHasher>,
        scale_factor: u32,
        iskips: u32,
        pskips: u32,
        dup_skips: u32,
        dup_pskips: u32,
        last_clean: u32,
    }

    impl<T: Ctx> Default for LimitedHeap<T> {
        fn default() -> LimitedHeap<T> {
            LimitedHeap::new()
        }
    }

    impl<T: Ctx> LimitedHeap<T> {
        fn score(ctx: &ContextWrapper<T>, scale_factor: u32) -> u32 {
            scale_factor * ctx.get().progress() * ctx.get().progress() + (1 << 28) - ctx.elapsed()
        }

        pub fn new() -> LimitedHeap<T> {
            LimitedHeap {
                max_time: u32::MAX,
                heap: {
                    let mut h = BinaryHeap::new();
                    h.reserve(2048);
                    h
                },
                states_seen: LruCache::with_hasher(
                    NonZeroUsize::new(1 << 23).unwrap(),
                    CommonHasher::default(),
                ),
                scale_factor: 50,
                iskips: 0,
                pskips: 0,
                dup_skips: 0,
                dup_pskips: 0,
                last_clean: 0,
            }
        }

        /// Returns the actual number of elements in the heap.
        /// Iterating over the heap may not produce this many elements.
        pub fn len(&self) -> usize {
            self.heap.len()
        }

        pub fn seen(&self) -> usize {
            self.states_seen.len()
        }

        pub fn scale_factor(&self) -> u32 {
            self.scale_factor
        }

        pub fn set_scale_factor(&mut self, factor: u32) {
            self.scale_factor = factor;
            if !self.heap.is_empty() {
                println!("Recalculating scores");
                self.clean()
            }
        }

        /// Returns whether the underlying heap is actually empty.
        /// Attempting to peek or pop may produce None instead.
        pub fn is_empty(&self) -> bool {
            self.heap.is_empty()
        }

        pub fn max_time(&self) -> u32 {
            self.max_time
        }

        pub fn set_max_time(&mut self, max_time: u32) {
            self.max_time = core::cmp::min(self.max_time, max_time);
        }

        pub fn set_lenient_max_time(&mut self, max_time: u32) {
            self.set_max_time(max_time + (max_time / 128))
        }

        /// Pushes an element into the heap.
        /// If the element's elapsed time is greater than the allowed maximum,
        /// or, the state has been previously seen with an equal or lower elapsed time, does nothing.
        pub fn push(&mut self, el: ContextWrapper<T>) {
            if el.elapsed() <= self.max_time {
                if let Some(min) = self.states_seen.get_mut(el.get()) {
                    if el.elapsed() < *min {
                        *min = el.elapsed();
                    } else {
                        self.dup_skips += 1;
                        return;
                    }
                } else {
                    self.states_seen.push(el.get().clone(), el.elapsed());
                }
                self.heap.push(HeapElement {
                    score: Self::score(&el, self.scale_factor),
                    el,
                });
            } else {
                self.iskips += 1;
            }
        }

        pub fn see(&mut self, el: &ContextWrapper<T>) -> bool {
            if el.elapsed() <= self.max_time {
                if let Some(min) = self.states_seen.get_mut(el.get()) {
                    if el.elapsed() < *min {
                        *min = el.elapsed();
                        true
                    } else {
                        self.dup_skips += 1;
                        false
                    }
                } else {
                    self.states_seen.push(el.get().clone(), el.elapsed());
                    true
                }
            } else {
                self.iskips += 1;
                false
            }
        }

        /// Returns the next element with the highest score, or None.
        /// Will skip over any elements whose elapsed time is greater than the allowed maximum,
        /// or whose elapsed time is greater than the minimum seen for that state.
        pub fn pop(&mut self) -> Option<ContextWrapper<T>> {
            // Lazily clear when the max time is changed with elements in the heap
            while let Some(el) = self.heap.pop() {
                if el.el.elapsed() <= self.max_time {
                    if let Some(&time) = self.states_seen.get(el.el.get()) {
                        if el.el.elapsed() <= time {
                            return Some(el.el);
                        } else {
                            self.dup_pskips += 1;
                        }
                    } else {
                        return Some(el.el);
                    }
                } else {
                    self.pskips += 1;
                }
            }
            None
        }

        /// Produces the actual first element of the heap.
        /// This may not be the element returned by pop().
        pub fn peek(&self) -> Option<&ContextWrapper<T>> {
            match self.heap.peek() {
                Some(el) => Some(&el.el),
                None => None,
            }
        }

        fn drain(&mut self) -> impl IntoIterator<Item = ContextWrapper<T>> + '_ {
            self.heap.drain().filter_map(|el| {
                if el.el.elapsed() <= self.max_time {
                    if let Some(&time) = self.states_seen.get(el.el.get()) {
                        if el.el.elapsed() <= time {
                            Some(el.el)
                        } else {
                            self.dup_pskips += 1;
                            None
                        }
                    } else {
                        Some(el.el)
                    }
                } else {
                    self.pskips += 1;
                    None
                }
            })
        }

        pub fn clean(&mut self) {
            println!("Cleaning... {}", self.heap.len());
            let start = std::time::Instant::now();
            let mut theap = BinaryHeap::new();
            self.heap.shrink_to_fit();
            theap.reserve(std::cmp::min(1048576, self.heap.len()));
            let factor = self.scale_factor;
            for el in self.drain() {
                theap.push(HeapElement {
                    score: Self::score(&el, factor),
                    el,
                });
            }
            self.heap = theap;
            let done = start.elapsed();
            println!("... -> {}. Done in {:?}.", self.heap.len(), done);
            self.last_clean = self.max_time;
        }

        pub fn extend<I>(&mut self, iter: I)
        where
            I: IntoIterator<Item = ContextWrapper<T>>,
        {
            self.heap.extend(iter.into_iter().filter_map(|c| {
                if let Some(min) = self.states_seen.get_mut(c.get()) {
                    if c.elapsed() < *min {
                        *min = c.elapsed();
                    } else {
                        self.dup_skips += 1;
                        return None;
                    }
                } else {
                    self.states_seen.push(c.get().clone(), c.elapsed());
                }
                if c.elapsed() <= self.max_time {
                    Some(HeapElement {
                        score: Self::score(&c, self.scale_factor),
                        el: c,
                    })
                } else {
                    self.iskips += 1;
                    None
                }
            }));
        }

        pub fn iter(&self) -> impl Iterator<Item = &ContextWrapper<T>> + '_ {
            self.heap.iter().filter_map(|e| {
                if e.el.elapsed() <= self.max_time {
                    if let Some(&time) = self.states_seen.peek(e.el.get()) {
                        if e.el.elapsed() <= time {
                            Some(&e.el)
                        } else {
                            None
                        }
                    } else {
                        Some(&e.el)
                    }
                } else {
                    None
                }
            })
        }

        pub fn stats(&self) -> (u32, u32, u32, u32) {
            (self.iskips, self.pskips, self.dup_skips, self.dup_pskips)
        }
    }

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
                            $ctx.spend(warp.price());
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
                            $crate::context::history_str::<$T, _>(
                                ctx.recent_history().iter().copied()
                            ),
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
}
