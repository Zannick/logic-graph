use crate::access::*;
use crate::context::*;
use crate::greedy::*;
use crate::heap::RocksBackedQueue;
use crate::minimize::*;
use crate::optimize::optimize;
use crate::solutions::SolutionCollector;
use crate::world::*;
use rayon::prelude::*;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum SearchMode {
    Classic,
    Greedy,
    MaxEstimate,
    MaxProgress,
    SomeProgress(usize),
    HalfProgress,
    Dependent,
    Single,
    Unknown,
}

fn mode_by_index(index: usize) -> SearchMode {
    match index % 16 {
        1 | 6 | 10 | 14 => SearchMode::Dependent,
        2 | 7 => SearchMode::Greedy,
        4 | 8 => SearchMode::MaxProgress,
        5 => SearchMode::MaxEstimate,
        9 => SearchMode::SomeProgress(1),
        11 => SearchMode::SomeProgress(2),
        13 => SearchMode::SomeProgress(4),
        15 => SearchMode::HalfProgress,
        // 0, 3, 12
        _ => SearchMode::Single,
    }
}

pub fn explore<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_time: u32,
) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<ExitId = L::ExitId, Context = T, Currency = L::Currency>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = L::Currency>,
{
    let spot_map = accessible_spots(world, ctx, max_time);
    let mut vec: Vec<ContextWrapper<T>> = spot_map.values().filter_map(Clone::clone).collect();

    vec.par_sort_unstable_by_key(|el| el.elapsed());
    vec
}

pub fn visit_locations<W, T, L, E>(world: &W, ctx: ContextWrapper<T>) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<ExitId = L::ExitId, Context = T, Currency = L::Currency>,
{
    let mut ctx_list = vec![ctx];
    let (mut locs, exit) = visitable_locations(world, ctx_list[0].get());
    if locs.is_empty() && exit.is_none() {
        return Vec::new();
    }
    locs.sort_unstable_by_key(|loc| loc.time());
    for loc in locs {
        let last_ctxs = ctx_list;
        ctx_list = Vec::new();
        ctx_list.reserve(last_ctxs.len() * 2);
        for mut ctx in last_ctxs {
            if ctx.get().todo(loc.id()) && loc.can_access(ctx.get()) {
                // Major branching factor: sometimes we can try skipping a location:
                // 1. If location has a cost, we might not want it.
                // 2. Otherwise, any location is potentially skippable.
                //    But it's not worth skipping locations that are free in time and money;
                //    they come along for free with other locations, or we route differently.
                if loc.time() > 0 || !loc.is_free() {
                    let mut newctx = ctx.clone();
                    newctx.get_mut().skip(loc.id());
                    // Check if this loc is required. If it is, we can't skip it.
                    if can_win_just_locations(world, newctx.get()) {
                        ctx_list.push(newctx);
                    }
                }

                // Get the item and mark the location visited.
                ctx.visit(world, loc);
            }
            ctx_list.push(ctx);
        }
    }

    if let Some(ExitWithLoc(l, e)) = exit {
        let exit = world.get_exit(e);
        let loc = world.get_location(l);
        for ctx in ctx_list.iter_mut() {
            if ctx.get().todo(l) && loc.can_access(ctx.get()) && exit.can_access(ctx.get()) {
                // Get the item and move along the exit.
                ctx.visit_exit(world, loc, exit);
            }
        }
    }
    ctx_list
}

pub fn activate_actions<W, T, L, E>(world: &W, ctx: &ContextWrapper<T>) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<ExitId = E::ExitId, Context = T>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
{
    let mut result = Vec::new();
    for act in world.get_global_actions() {
        if act.can_access(ctx.get()) {
            let mut c2 = ctx.clone();
            c2.activate(act);
            if c2.get() != ctx.get() {
                result.push(c2);
            }
        }
    }
    for act in world.get_spot_actions(ctx.get().position()) {
        if act.can_access(ctx.get()) {
            let mut c2 = ctx.clone();
            c2.activate(act);
            if c2.get() != ctx.get() {
                result.push(c2);
            }
        }
    }
    result
}

pub fn classic_step<W, T, L>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_time: u32,
) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    // The process will look more like this:
    // 1. explore -> vec of spot ctxs with penalties applied
    // 2. get largest dist
    // 3. (activate_actions) for each ctx, check for global actions and spot actions
    // 4. (visit_locations) for each ctx, get all available locations
    let spot_ctxs = explore(world, ctx, max_time);
    let mut result = Vec::new();

    if !spot_ctxs.is_empty() {
        let spots: Vec<_> = spot_ctxs
            .into_iter()
            .map(|ctx| (spot_has_locations(world, ctx.get()), ctx))
            .collect();
        for (has_locs, ctx) in spots {
            if spot_has_actions(world, ctx.get()) {
                result.extend(activate_actions(world, &ctx));
            }
            if has_locs {
                result.extend(visit_locations(world, ctx));
            }
        }
    }
    result
}

pub fn single_step<W, T, L>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_time: u32,
) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    // One movement total
    let movement_state = ctx.get().get_movement_state();
    let mut results = Vec::new();
    for ce in world.get_condensed_edges_from(ctx.get().position()) {
        if ce.time + ctx.elapsed() <= max_time && ce.can_access(world, ctx.get(), movement_state) {
            let mut newctx = ctx.clone();
            newctx.move_condensed_edge(ce);
            results.push(newctx);
        }
    }
    for exit in world.get_spot_exits(ctx.get().position()) {
        if exit.time() + ctx.elapsed() <= max_time && exit.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.exit(exit);
            results.push(newctx);
        }
    }
    for warp in world.get_warps() {
        if warp.time() + ctx.elapsed() <= max_time && warp.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.warp(warp);
            results.push(newctx);
        }
    }
    results.extend(activate_actions(world, &ctx));
    // This can technically do more than one location at a time, but that's fine I guess
    results.extend(visit_locations(world, ctx));
    results
}

pub struct Search<'a, W, T>
where
    W: World,
    T: Ctx<World = W> + Debug,
{
    world: &'a W,
    startctx: ContextWrapper<T>,
    solutions: Mutex<SolutionCollector<T>>,
    queue: RocksBackedQueue<'a, W, T>,
    iters: AtomicUsize,
    extras: AtomicU64,
    deadends: AtomicU32,
    optimizes: AtomicU32,
}

impl<'a, W, T, L, E> Search<'a, W, T>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<Context = T, ExitId = L::ExitId, LocId = L::LocId, Currency = L::Currency>,
{
    fn find_greedy_win(world: &W, startctx: &ContextWrapper<T>) -> ContextWrapper<T> {
        let start = Instant::now();
        match greedy_search(world, &startctx, u32::MAX) {
            Ok(wonctx) => {
                println!(
                    "Finished greedy search in {:?} with a result of {}ms",
                    start.elapsed(),
                    wonctx.elapsed()
                );
                wonctx
            }
            Err(ctx) => {
                panic!(
                    "Found no greedy solution, maximal attempt reached dead-end after {}ms:\n{}\n{:#?}",
                    ctx.elapsed(),
                    ctx.history_summary(),
                    ctx.get()
                );
            }
        }
    }

    pub fn new(
        world: &'a W,
        mut ctx: T,
        routes: Vec<ContextWrapper<T>>,
    ) -> Result<Search<'a, W, T>, std::io::Error> {
        world.skip_unused_items(&mut ctx);

        let startctx = ContextWrapper::new(ctx);
        let mut solutions = SolutionCollector::<T>::new("data/solutions.txt", "data/previews.txt")?;

        let mut intermediate = Vec::new();

        let mut wins = Vec::new();
        let mut others = 0;
        for c in routes {
            let hist: Vec<_> = c.history_rev().collect();
            let mut newctx = startctx.clone();
            for h in hist.iter().rev() {
                newctx.replay(world, *h);
                // We want to at least remember each intermediate state in the queue.
                if !world.won(newctx.get()) {
                    intermediate.push(newctx.clone());
                }
            }
            if world.won(c.get()) {
                wins.push(c);
            } else {
                // We don't need to save the others, since we recreated them above.
                others += 1;
            }
        }

        wins.sort_unstable_by_key(|c| !c.elapsed());

        if !wins.is_empty() {
            println!(
                "Provided extra routes: {} winners, {} not\nwinning times: {:?}",
                wins.len(),
                others,
                wins.iter().map(|c| c.elapsed()).collect::<Vec<_>>()
            );
        } else if others > 0 {
            println!(
                "Provided {} non-winning routes, performing greedy search...",
                others
            );
        } else {
            println!("No routes provided, performing greedy search...");
        }
        let wonctx = wins
            .pop()
            .unwrap_or_else(|| Self::find_greedy_win(world, &startctx));

        let start = Instant::now();
        let (max_time, clean_ctx) = if let Some(m) =
            minimize_greedy(world, startctx.get(), &wonctx, wonctx.elapsed())
        {
            println!("Minimized in {:?}", start.elapsed());
            println!(
                "Initial solution of {}ms was minimized to {}ms",
                wonctx.elapsed(),
                m.elapsed()
            );
            let max_time = std::cmp::min(wonctx.elapsed(), m.elapsed());
            let clean_ctx = ContextWrapper::new(remove_all_unvisited(world, startctx.get(), &m));
            solutions.insert(m);
            (max_time, clean_ctx)
        } else {
            println!("Minimized-greedy solution wasn't faster than original");
            (
                wonctx.elapsed(),
                ContextWrapper::new(remove_all_unvisited(world, startctx.get(), &wonctx)),
            )
        };

        solutions.insert(wonctx);
        for w in wins {
            solutions.insert(w);
        }

        let queue = RocksBackedQueue::new(
            ".db",
            world,
            &startctx,
            max_time + max_time / 10,
            1_048_576,
            32_768,
            262_144,
            1_024,
            32_768,
        )
        .unwrap();
        queue.push(startctx.clone()).unwrap();
        queue.push(clean_ctx).unwrap();
        queue.extend(intermediate).unwrap();
        println!("Max time to consider is now: {}ms", queue.max_time());
        println!("Queue starts with {} elements", queue.len());
        Ok(Search {
            world,
            startctx,
            solutions: Mutex::new(solutions),
            queue,
            iters: 0.into(),
            extras: 0.into(),
            deadends: 0.into(),
            optimizes: 0.into(),
        })
    }

    fn handle_solution(&self, ctx: ContextWrapper<T>, mode: SearchMode) {
        let old_time = self.queue.max_time();
        let iters = self.iters.load(Ordering::Acquire);
        let mut sols = self.solutions.lock().unwrap();
        if iters > 10_000_000 && sols.unique() > 4 {
            self.queue.set_max_time(ctx.elapsed());
        } else {
            self.queue.set_lenient_max_time(ctx.elapsed());
        }

        if sols.is_empty() || ctx.elapsed() < sols.best() {
            println!(
                "{:?} mode found new shortest winning path after {} rounds: estimated {}ms (heap max was: {}ms)",
                mode,
                iters,
                ctx.elapsed(),
                old_time
            );
            println!("Max time to consider is now: {}ms", self.queue.max_time());
        }

        // If there were locations we skipped mid-route, skip them from the start,
        // in case that changes the routing.
        let newctx =
            ContextWrapper::new(remove_all_unvisited(self.world, self.startctx.get(), &ctx));
        self.queue.push(newctx).unwrap();

        let max_time = ctx.elapsed();
        if let Some(unique_history) = sols.insert(ctx) {
            drop(sols);
            println!("Found new unique solution");
            let start = Instant::now();
            let mut opt = optimize(
                self.queue.db(),
                self.world,
                self.startctx.get(),
                unique_history,
            );
            if let Some(best) = opt.pop() {
                if self.world.won(best.get()) {
                    if best.elapsed() < max_time {
                        println!(
                            "Optimized this type of solution to {}ms in {:?}",
                            best.elapsed(),
                            start.elapsed()
                        );
                        self.optimizes.fetch_add(1, Ordering::Release);
                        self.handle_solution(best, mode);
                    }
                } else {
                    opt.push(best);
                }
                self.queue.extend(opt).unwrap();
            }
        }
    }

    fn handle_greedy_solution(
        &self,
        ctx: ContextWrapper<T>,
        fork: &ContextWrapper<T>,
        mode: SearchMode,
    ) {
        // Create intermediate states to add to the queue.
        let mut winhist: Vec<_> = ctx.history_rev().collect();
        let oldhist: Vec<_> = fork.history_rev().collect();
        let oldhistlen = oldhist.len();
        winhist.truncate(winhist.len() - oldhistlen);

        let mut newstates = Vec::new();
        let mut stepping = fork.clone();
        for step in winhist.into_iter().rev() {
            stepping.replay(self.world, step);
            if !matches!(step, History::Move(_) | History::MoveLocal(_)) {
                newstates.push(stepping.clone());
            }
        }

        let mstart = Instant::now();
        if let Some(m) = minimize_greedy(self.world, self.startctx.get(), &ctx, ctx.elapsed()) {
            println!(
                "Minimized greedy solution from {}ms -> {}ms in {:?}",
                ctx.elapsed(),
                m.elapsed(),
                mstart.elapsed()
            );
            self.handle_solution(m, mode);
        }
        self.handle_solution(ctx, mode);

        if let Some(last) = newstates.last() {
            let mut hist: Vec<_> = last.history_rev().collect();
            hist.reverse();
            let mut rebuilt = Vec::with_capacity(newstates.len());
            let mut replay = self.startctx.clone();
            for step in hist {
                replay.replay(self.world, step);
                if matches!(step, History::Get(..) | History::MoveGet(..)) {
                    rebuilt.push(replay.clone());
                }
            }
            self.queue.extend(rebuilt).unwrap();
        }
    }

    fn extract_solutions(
        &self,
        states: Vec<ContextWrapper<T>>,
        mode: SearchMode,
    ) -> Vec<ContextWrapper<T>> {
        states
            .into_iter()
            .filter_map(|ctx| {
                if self.world.won(ctx.get()) {
                    self.handle_solution(ctx, mode);
                    None
                } else {
                    Some(ctx)
                }
            })
            .collect()
    }

    fn classic_step(&self, ctx: ContextWrapper<T>) -> Vec<ContextWrapper<T>> {
        classic_step(self.world, ctx, self.queue.max_time())
    }

    fn single_step(&self, ctx: ContextWrapper<T>) -> Vec<ContextWrapper<T>> {
        single_step(self.world, ctx, self.queue.max_time())
    }

    fn choose_mode(&self, iters: usize, _ctx: &ContextWrapper<T>) -> SearchMode {
        if iters % 1024 != 0 {
            SearchMode::Single
        } else if iters % 2048 != 0 {
            SearchMode::Greedy
        } else {
            SearchMode::SomeProgress((iters / 4096) % 32)
        }
    }

    pub fn search(self) -> Result<(), std::io::Error> {
        let finished = AtomicBool::new(false);
        let start = Mutex::new(Instant::now());
        println!(
            "Starting search with {} threads",
            rayon::current_num_threads()
        );

        struct Iter<'a, W, T>
        where
            W: World,
            T: Ctx<World = W>,
        {
            q: &'a RocksBackedQueue<'a, W, T>,
        }
        impl<'a, W, T> Iterator for Iter<'a, W, T>
        where
            W: World,
            T: Ctx<World = W>,
            W::Location: Accessible<Context = T>,
            W::Warp: Accessible<Context = T>,
        {
            type Item = ContextWrapper<T>;

            fn next(&mut self) -> Option<Self::Item> {
                self.q.pop().unwrap()
            }
        }

        let res = rayon::scope(|scope| {
            scope.spawn(|_| {
                let sleep_time = Duration::from_secs(10);
                while !finished.load(Ordering::Acquire) {
                    let len = self.queue.db_len();
                    if len < 1_000_000 {
                        sleep(sleep_time);
                        continue;
                    }
                    self.queue.db_cleanup(65_536).unwrap();
                }
            });

            let mut res = Ok(());
            while res.is_ok() && !self.queue.is_empty() {
                let iter = Iter { q: &self.queue };
                res = iter.par_bridge().try_for_each(|item| {
                    let vec = self.process_one(
                        item,
                        &start,
                        mode_by_index(rayon::current_thread_index().unwrap_or_default()),
                    )?;
                    if !vec.is_empty() {
                        self.queue.extend(vec).map(|_| ())
                    } else {
                        Ok(())
                    }
                });
            }
            finished.store(true, Ordering::Release);
            res
        });
        let (iskips, pskips, dskips, dpskips) = self.queue.skip_stats();
        println!(
            "Finished after {} rounds ({} dead-ends), skipped {}+{} pushes + {}+{} pops: {}",
            self.iters.load(Ordering::Acquire),
            self.deadends.load(Ordering::Acquire),
            iskips,
            dskips,
            pskips,
            dpskips,
            match res {
                Ok(_) => String::from("emptied queue"),
                Err(s) => s,
            }
        );
        self.queue.print_queue_histogram();
        self.solutions.into_inner().unwrap().export()
    }

    fn process_one(
        &self,
        ctx: ContextWrapper<T>,
        start: &Mutex<Instant>,
        mode: SearchMode,
    ) -> Result<Vec<ContextWrapper<T>>, &str> {
        let iters = self.iters.fetch_add(1, Ordering::AcqRel) + 1;
        if ctx.get().count_visits() + ctx.get().count_skips() >= W::NUM_LOCATIONS {
            if self.world.won(ctx.get()) {
                self.handle_solution(ctx, SearchMode::Unknown);
            } else {
                self.deadends.fetch_add(1, Ordering::Release);
            }
            return Ok(Vec::new());
        }

        if iters % 100_000 == 0 {
            self.print_status_update(&start, iters, 100_000, &ctx);
        }

        let current_mode = if iters < 500_000 {
            SearchMode::Single
        } else if mode == SearchMode::Dependent {
            self.choose_mode(iters, &ctx)
        } else {
            mode
        };
        match current_mode {
            SearchMode::Single => Ok(self.extract_solutions(self.single_step(ctx), current_mode)),
            SearchMode::Greedy => {
                // Run a classic step on the given state and handle any solutions
                let next = self.extract_solutions(self.classic_step(ctx), SearchMode::Classic);
                // Pick a state greedily: max progress, min elapsed, and do a greedy search.
                if let Some(ctx) = next
                    .iter()
                    .max_by_key(|ctx| (self.queue.db().progress(ctx.get()), !ctx.elapsed()))
                {
                    if let Ok(win) = greedy_search(self.world, &ctx, self.queue.max_time()) {
                        if win.elapsed() <= self.queue.max_time() {
                            self.handle_greedy_solution(win, &ctx, mode);
                        }
                    }
                }

                // All the classic states are still pushed to the queue, even the one we used
                self.extras.fetch_add(1, Ordering::Release);
                Ok(next)
            }
            SearchMode::MaxEstimate => {
                let mut next = self.extract_solutions(self.classic_step(ctx), SearchMode::Classic);
                if let Some(ctx2) = self.queue.pop_max_estimate().unwrap() {
                    next.extend(
                        self.extract_solutions(self.single_step(ctx2), SearchMode::MaxEstimate),
                    );
                }
                self.extras.fetch_add(1, Ordering::Release);
                Ok(next)
            }
            SearchMode::MaxProgress => {
                let mut next = self.extract_solutions(self.single_step(ctx), SearchMode::Single);
                if let Some(ctx2) = self.queue.pop_max_progress().unwrap() {
                    next.extend(self.extract_solutions(self.single_step(ctx2), current_mode));
                }
                self.extras.fetch_add(1, Ordering::Release);
                Ok(next)
            }
            SearchMode::SomeProgress(p) => {
                let mut next = self.extract_solutions(self.single_step(ctx), SearchMode::Single);
                if let Some(ctx2) = self.queue.pop_min_progress(p).unwrap() {
                    next.extend(self.extract_solutions(self.single_step(ctx2), current_mode));
                }
                self.extras.fetch_add(1, Ordering::Release);
                Ok(next)
            }
            SearchMode::HalfProgress => {
                let mut next = self.extract_solutions(self.single_step(ctx), SearchMode::Single);
                if let Some(ctx2) = self.queue.pop_half_progress().unwrap() {
                    next.extend(self.extract_solutions(self.single_step(ctx2), current_mode));
                }
                self.extras.fetch_add(1, Ordering::Release);
                Ok(next)
            }
            _ => {
                let next = self.extract_solutions(self.classic_step(ctx), SearchMode::Classic);
                Ok(next)
            }
        }
    }

    fn print_status_update(
        &self,
        start: &Mutex<Instant>,
        iters: usize,
        num_rounds: u32,
        ctx: &ContextWrapper<T>,
    ) {
        let mut s = start.lock().unwrap();
        println!("{} iters took {:?}", num_rounds, s.elapsed());
        *s = Instant::now();

        let sols = self.solutions.lock().unwrap();
        if iters > 10_000_000 && sols.unique() > 4 {
            self.queue.set_max_time(sols.best());
        }
        if iters % 1_000_000 == 0 {
            self.queue.print_queue_histogram();
        }
        let (iskips, pskips, dskips, dpskips) = self.queue.skip_stats();
        let max_time = self.queue.max_time();
        let (arc, arccount) = self.queue.db().archive_stats();
        println!(
            "--- Round {} (ex: {}, solutions: {}, unique: {}, dead-ends={}; opt={}) ---\n\
            Stats: heap={}; db={}; total={}; seen={}; estimates={}; cached={}\n\
            limit={}ms; db best={}; archived={}/{}; evictions={}; retrievals={}\n\
            skips: push:{} time, {} dups; pop: {} time, {} dups; bgdel={}\n\
            {}",
            iters,
            self.extras.load(Ordering::Acquire),
            sols.len(),
            sols.unique(),
            self.deadends.load(Ordering::Acquire),
            self.optimizes.load(Ordering::Acquire),
            self.queue.heap_len(),
            self.queue.db_len(),
            self.queue.len(),
            self.queue.seen(),
            self.queue.estimates(),
            self.queue.cached_estimates(),
            max_time,
            self.queue.db_best(),
            arc,
            arccount,
            self.queue.evictions(),
            self.queue.retrievals(),
            iskips,
            dskips,
            pskips,
            dpskips,
            self.queue.background_deletes(),
            ctx.info(self.queue.estimated_remaining_time(ctx))
        );
    }
}
