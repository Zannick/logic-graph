use crate::access::*;
use crate::context::*;
use crate::greedy::*;
use crate::heap::RocksBackedQueue;
use crate::solutions::SolutionCollector;
use crate::world::*;
use anyhow::Result;
use rayon::prelude::*;
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum SearchMode {
    Standard,
    MaxProgress,
    SomeProgress(usize),
    HalfProgress,
    Dependent,
    Unknown,
}

fn mode_by_index(index: usize) -> SearchMode {
    match index % 16 {
        1 | 6 | 10 | 14 => SearchMode::Dependent,
        2 | 4 | 5 => SearchMode::MaxProgress,
        9 => SearchMode::SomeProgress(1),
        11 => SearchMode::SomeProgress(2),
        13 => SearchMode::SomeProgress(4),
        15 => SearchMode::HalfProgress,
        // 0, 3, 7, 8, 12
        _ => SearchMode::Standard,
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
                    history_summary::<T>(ctx.recent_history()),
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

        let mut wins = Vec::new();
        let mut others = 0;
        for c in routes {
            if world.won(c.get()) {
                wins.push(c);
            } else {
                // We don't save the others at all.
                // (No intermediate states are created to avoid history duplication.)
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
        let max_time =
            if let Some(m) = minimize_greedy(world, startctx.get(), &wonctx, wonctx.elapsed()) {
                println!("Minimized in {:?}", start.elapsed());
                println!(
                    "Initial solution of {}ms was minimized to {}ms",
                    wonctx.elapsed(),
                    m.elapsed()
                );
                let max_time = std::cmp::min(wonctx.elapsed(), m.elapsed());
                solutions.insert(
                    m.elapsed(),
                    m.recent_history().into_iter().copied().collect(),
                );
                max_time
            } else {
                println!("Minimized-greedy solution wasn't faster than original");
                wonctx.elapsed()
            };

        solutions.insert(
            wonctx.elapsed(),
            wonctx.recent_history().into_iter().copied().collect(),
        );
        for w in wins {
            solutions.insert(
                w.elapsed(),
                w.recent_history().into_iter().copied().collect(),
            );
        }

        let queue = RocksBackedQueue::new(
            ".db",
            world,
            &startctx,
            max_time + max_time / 128,
            1_048_576,
            131_072,
            262_144,
            1_024,
            32_768,
        )
        .unwrap();
        queue.push(startctx.clone(), &None).unwrap();
        println!("Max time to consider is now: {}ms", queue.max_time());
        println!("Queue starts with {} elements", queue.len());
        Ok(Search {
            world,
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

        if let Some(_) = sols.insert(
            ctx.elapsed(),
            self.queue.db().get_history_ctx(&ctx).unwrap(),
        ) {
            drop(sols);
            println!("Found new unique solution");
        }
    }

    fn extract_solutions(
        &self,
        states: Vec<ContextWrapper<T>>,
        mode: SearchMode,
    ) -> Vec<ContextWrapper<T>> {
        let max_time = self.queue.max_time();
        states
            .into_iter()
            .filter_map(|ctx| {
                if ctx.elapsed() > max_time {
                    None
                } else if self.world.won(ctx.get()) {
                    self.handle_solution(ctx, mode);
                    None
                } else {
                    Some(ctx)
                }
            })
            .collect()
    }

    fn single_step(&self, ctx: ContextWrapper<T>) -> Vec<ContextWrapper<T>> {
        single_step(self.world, ctx, self.queue.max_time())
    }

    fn choose_mode(&self, iters: usize) -> SearchMode {
        match iters % 8 {
            0 => SearchMode::SomeProgress((iters / 8) % 32),
            1 => SearchMode::MaxProgress,
            2 => SearchMode::HalfProgress,
            3 => SearchMode::SomeProgress(5),

            _ => SearchMode::Standard,
        }
    }

    pub fn search(self) -> Result<(), std::io::Error> {
        let finished = AtomicBool::new(false);
        let threads_done = AtomicUsize::new(0);
        let start = Mutex::new(Instant::now());
        let num_threads = rayon::current_num_threads();
        let res = Mutex::new(Ok(()));
        println!("Starting search with {} threads", num_threads);

        let run_thread = |i| {
            let mode = mode_by_index(i);
            let mut done = false;
            while !finished.load(Ordering::Acquire)
                && threads_done.load(Ordering::Acquire) < num_threads
            {
                let iters = self.iters.load(Ordering::Acquire);
                let current_mode = if iters < 500_000 {
                    SearchMode::Standard
                } else if mode == SearchMode::Dependent {
                    self.choose_mode(iters)
                } else {
                    mode
                };

                let items = match current_mode {
                    SearchMode::MaxProgress => self.queue.pop_max_progress(2),
                    SearchMode::HalfProgress => self.queue.pop_half_progress(2),
                    SearchMode::SomeProgress(p) => self.queue.pop_min_progress(p, 2),
                    _ => self.queue.pop_round_robin(),
                };
                match items {
                    Ok(items) => {
                        if items.is_empty() {
                            if !done {
                                done = true;
                                threads_done.fetch_add(1, Ordering::Release);
                            }
                            sleep(Duration::from_secs(1));
                            continue;
                        }

                        if done {
                            threads_done.fetch_sub(1, Ordering::Acquire);
                            done = false;
                        }

                        for ctx in items {
                            let iters = self.iters.fetch_add(1, Ordering::AcqRel) + 1;
                            let prev = ctx.get().clone();
                            if let Err(e) = self
                                .queue
                                .extend(self.process_one(ctx, iters, &start, mode), &Some(prev))
                            {
                                let mut r = res.lock().unwrap();
                                println!("Thread {} exiting due to error: {:?}", i, e);
                                if r.is_ok() {
                                    *r = Err(e);
                                    finished.store(true, Ordering::Release);
                                }
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Thread {} exiting due to error: {:?}", i, e);
                        let mut r = res.lock().unwrap();
                        if r.is_ok() {
                            *r = Err(e);
                            finished.store(true, Ordering::Release);
                        }
                        return;
                    }
                };
            }
            println!(
                "Thread {} exiting: fin={} done={}",
                i,
                finished.load(Ordering::Acquire),
                threads_done.load(Ordering::Acquire)
            );
        };

        rayon::scope(|scope| {
            scope.spawn(|_| {
                let sleep_time = Duration::from_secs(10);
                while !finished.load(Ordering::Acquire) {
                    let len = self.queue.db_len();
                    if len < 1_000_000 {
                        sleep(sleep_time);
                        continue;
                    }
                    self.queue.db_cleanup(65_536, &finished).unwrap();
                }
            });

            rayon::scope(|sc2| {
                for i in 0..num_threads {
                    sc2.spawn(move |_| run_thread(i));
                }
            });

            println!("Workers all exited, marking finished");
            finished.store(true, Ordering::Release);
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
            match res.into_inner().unwrap() {
                Ok(_) => String::from("emptied queue"),
                Err(s) => s.to_string(),
            }
        );
        self.queue.print_queue_histogram();
        self.solutions.into_inner().unwrap().export()
    }

    fn process_one(
        &self,
        ctx: ContextWrapper<T>,
        iters: usize,
        start: &Mutex<Instant>,
        mode: SearchMode,
    ) -> Vec<ContextWrapper<T>> {
        if iters % 100_000 == 0 {
            self.print_status_update(&start, iters, 100_000, &ctx);
        }

        if ctx.get().count_visits() + ctx.get().count_skips() >= W::NUM_LOCATIONS {
            if self.world.won(ctx.get()) {
                self.handle_solution(ctx, SearchMode::Unknown);
            } else {
                self.deadends.fetch_add(1, Ordering::Release);
            }
            return Vec::new();
        }

        self.extract_solutions(self.single_step(ctx), mode)
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
        println!(
            "--- Round {} (ex: {}, solutions: {}, unique: {}, dead-ends={}; opt={}) ---\n\
            Stats: heap={}; db={}; total={}; seen={}; estimates={}; cached={}\n\
            limit={}ms; db best={}; evictions={}; retrievals={}\n\
            skips: push:{} time, {} dups; pop: {} time, {} dups; bgdel={}\n\
            db bests: {}\n\
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
            self.queue.evictions(),
            self.queue.retrievals(),
            iskips,
            dskips,
            pskips,
            dpskips,
            self.queue.background_deletes(),
            self.queue
                .db_bests()
                .into_iter()
                .map(|n| if n < u32::MAX {
                    n.to_string()
                } else {
                    String::from("-")
                })
                .collect::<Vec<_>>()
                .join(", "),
            ctx.info(
                self.queue.estimated_remaining_time(ctx),
                self.queue.db().progress(ctx.get()),
                self.queue.db().get_last_history_step(ctx).unwrap()
            )
        );
    }
}
