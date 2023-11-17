use crate::access::*;
use crate::context::*;
use crate::greedy::*;
use crate::heap::RocksBackedQueue;
use crate::minimize::pinpoint_minimize;
use crate::solutions::SolutionCollector;
use crate::world::*;
use anyhow::Result;
use log;
use rayon::prelude::*;
use std::fmt::Debug;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum SearchMode {
    Standard,
    MaxProgress(usize),
    SomeProgress(usize),
    HalfProgress,
    Dependent,
    LocalMinima,
    Greedy,
    Start,
    Minimized,
    Unknown,
}

fn mode_by_index(index: usize) -> SearchMode {
    match index % 16 {
        1 => SearchMode::Greedy,
        6 | 10 | 14 => SearchMode::Dependent,
        5 => SearchMode::MaxProgress(4),
        11 => SearchMode::SomeProgress(3),
        2 | 3 | 13 | 15 => SearchMode::LocalMinima,
        // 0, 4, 7, 8, 9, 12
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
    let mut vec: Vec<ContextWrapper<T>> = spot_map.into_values().collect();

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
    let (mut locs, exit) = visitable_locations(world, ctx.get());
    if locs.is_empty() && exit.is_none() {
        return Vec::new();
    }
    locs.sort_unstable_by_key(|loc| loc.time());
    let mut result = Vec::new();
    for loc in locs {
        if ctx.get().todo(loc.id()) && loc.can_access(ctx.get()) {
            // Get the item and mark the location visited.
            let mut newctx = ctx.clone();
            newctx.visit(world, loc);
            result.push(newctx);
        }
    }

    if let Some(ExitWithLoc(l, e)) = exit {
        let exit = world.get_exit(e);
        let loc = world.get_location(l);
        if ctx.get().todo(l) && loc.can_access(ctx.get()) && exit.can_access(ctx.get()) {
            // Get the item and move along the exit.
            let mut newctx = ctx.clone();
            newctx.visit_exit(world, loc, exit);
            result.push(newctx);
        }
    }
    result
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

fn single_step_with_local<W, T, L>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_time: u32,
) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    let movement_state = ctx.get().get_movement_state();
    let mut results = Vec::new();
    for &dest in world.get_area_spots(ctx.get().position()) {
        let ltt = ctx.get().local_travel_time(movement_state, dest);
        if ltt < u32::MAX && ltt + ctx.elapsed() <= max_time {
            let mut newctx = ctx.clone();
            newctx.move_local(dest, ltt);
            results.push(newctx);
        }
    }
    results.extend(single_step(world, ctx, max_time));
    results
}

pub struct Search<'a, W, T>
where
    W: World,
    T: Ctx<World = W> + Debug,
{
    world: &'a W,
    startctx: ContextWrapper<T>,
    solutions: Arc<Mutex<SolutionCollector<T>>>,
    queue: RocksBackedQueue<'a, W, T>,
    iters: AtomicUsize,
    deadends: AtomicU32,
    greedies: AtomicUsize,
    held: AtomicUsize,
    organic_solution: AtomicBool,
    organic_level: AtomicUsize,
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
        match greedy_search(world, startctx, u32::MAX, 9) {
            Ok(wonctx) => {
                log::info!(
                    "Finished greedy search in {:?} with a result of {}ms",
                    start.elapsed(),
                    wonctx.elapsed()
                );
                wonctx
            }
            Err(ctx) => {
                panic!(
                    "Found no greedy solution, maximal attempt reached dead-end after {}ms:\n{}\n{:#?}\nMissing: {:?}",
                    ctx.elapsed(),
                    history_summary::<T, _>(ctx.recent_history().iter().copied()),
                    ctx.get(),
                    world.items_needed(ctx.get())
                );
            }
        }
    }

    pub fn new<P>(
        world: &'a W,
        mut ctx: T,
        routes: Vec<ContextWrapper<T>>,
        db_path: P,
    ) -> Result<Search<'a, W, T>, std::io::Error>
    where
        P: AsRef<Path>,
    {
        world.skip_unused_items(&mut ctx);

        let startctx = ContextWrapper::new(ctx);
        let mut solutions = SolutionCollector::<T>::new(
            "data/solutions.txt",
            "data/previews.txt",
            "data/best.txt",
        )?;

        let mut wins = Vec::new();
        let mut others = Vec::new();
        for c in routes {
            if world.won(c.get()) {
                wins.push(c);
            } else {
                others.push(c)
            }
        }

        wins.sort_unstable_by_key(|c| !c.elapsed());

        if !wins.is_empty() {
            log::info!(
                "Provided extra routes: {} winners, {} not\nwinning times: {:?}",
                wins.len(),
                others.len(),
                wins.iter().map(|c| c.elapsed()).collect::<Vec<_>>()
            );
        } else if !others.is_empty() {
            log::info!(
                "Provided {} non-winning routes, performing greedy search...",
                others.len()
            );
        } else {
            log::info!("No routes provided, performing greedy search...");
        }
        let wonctx = wins
            .pop()
            .unwrap_or_else(|| Self::find_greedy_win(world, &startctx));

        let start = Instant::now();
        let max_time =
            if let Some(m) = minimize_greedy(world, startctx.get(), &wonctx, wonctx.elapsed()) {
                log::info!("Minimized in {:?}", start.elapsed());
                log::info!(
                    "Initial solution of {}ms was minimized to {}ms",
                    wonctx.elapsed(),
                    m.elapsed()
                );
                let max_time = std::cmp::min(wonctx.elapsed(), m.elapsed());
                solutions.insert(m.elapsed(), m.recent_history().to_vec());
                max_time
            } else {
                log::info!("Minimized-greedy solution wasn't faster than original");
                wonctx.elapsed()
            };

        solutions.insert(wonctx.elapsed(), wonctx.recent_history().to_vec());
        for w in &wins {
            solutions.insert(w.elapsed(), w.recent_history().to_vec());
        }

        let solutions = Arc::new(Mutex::new(solutions));

        let queue = RocksBackedQueue::new(
            db_path,
            world,
            &startctx,
            max_time + max_time / 128,
            1_048_576,
            262_144,
            524_288,
            4_096,
            32_768,
            solutions.clone(),
        )
        .unwrap();
        queue.push(startctx.clone(), &None).unwrap();
        log::info!("Max time to consider is now: {}ms", queue.max_time());
        let s = Search {
            world,
            startctx,
            solutions,
            queue,
            iters: 0.into(),
            deadends: 0.into(),
            held: 0.into(),
            greedies: 0.into(),
            organic_solution: false.into(),
            organic_level: 0.into(),
        };
        s.recreate_store(&s.startctx, wonctx.recent_history(), SearchMode::Start)
            .unwrap();
        for w in wins {
            s.recreate_store(&s.startctx, w.recent_history(), SearchMode::Start)
                .unwrap();
        }
        for o in others {
            s.recreate_store(&s.startctx, o.recent_history(), SearchMode::Start)
                .unwrap();
        }
        log::info!("Queue starts with {} elements", s.queue.len());
        Ok(s)
    }

    fn handle_solution(&self, ctx: &mut ContextWrapper<T>, prev: &Option<T>, mode: SearchMode) {
        // If prev is None we don't know the prev state
        // but also we should have no recent history in ctx.
        // But if prev is true, we must only record the state, since
        // recording `next` requires all the states at once.
        if prev.is_some() {
            self.queue.db().record_one(ctx, prev, true).unwrap();
        }

        self.organic_solution.store(true, Ordering::Release);

        let mut old_time = self.queue.max_time();
        let iters = self.iters.load(Ordering::Acquire);

        let history = self.queue.db().get_history(ctx.get()).unwrap();
        let elapsed = self.queue.db().get_best_elapsed(ctx.get()).unwrap();
        log::info!("Recording solution from {:?} mode: {}ms", mode, elapsed);

        let min_ctx = pinpoint_minimize(self.world, self.startctx.get(), &history);

        let mut sols = self.solutions.lock().unwrap();
        if iters > 10_000_000 && sols.unique() > 4 {
            self.queue.set_max_time(elapsed);
        } else {
            self.queue.set_lenient_max_time(elapsed);
        }

        if sols.is_empty() || elapsed < sols.best() {
            log::info!(
                "{:?} mode found new shortest winning path after {} rounds: estimated {}ms (heap max was: {}ms)",
                mode,
                iters,
                elapsed,
                old_time
            );
            old_time = self.queue.max_time();
            log::info!("Max time to consider is now: {}ms", old_time);
        }

        if sols.insert(elapsed, history).is_some() {
            log::info!("{:?} mode found new unique solution", mode);
        }

        if let Some(ctx) = min_ctx {
            if iters > 10_000_000 && sols.unique() > 4 {
                self.queue.set_max_time(ctx.elapsed());
            } else {
                self.queue.set_lenient_max_time(ctx.elapsed());
            }

            if ctx.elapsed() < sols.best() {
                log::info!(
                    "{:?} minimized a better solution: estimated {}ms (heap max was: {}ms)",
                    mode,
                    ctx.elapsed(),
                    old_time
                );
                log::info!("Max time to consider is now: {}ms", self.queue.max_time());
            }

            let history = ctx.recent_history();
            if sols.insert(ctx.elapsed(), history.to_vec()).is_some() {
                log::info!("Minimized found new unique solution");
            }

            drop(sols);
            self.recreate_store(&self.startctx, history, SearchMode::Minimized)
                .unwrap();
        }
    }

    fn extract_solutions(
        &self,
        states: Vec<ContextWrapper<T>>,
        prev: &Option<T>,
        mode: SearchMode,
    ) -> Vec<ContextWrapper<T>> {
        let max_time = self.queue.max_time();
        states
            .into_iter()
            .filter_map(|mut ctx| {
                if ctx.elapsed() > max_time {
                    None
                } else if self.world.won(ctx.get()) {
                    self.handle_solution(&mut ctx, prev, mode);
                    // The state is added to the db in handle_solution
                    // and the ctx no longer has history attached.
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

    fn recreate_step(&self, ctx: ContextWrapper<T>) -> Vec<ContextWrapper<T>> {
        single_step_with_local(self.world, ctx, u32::MAX)
    }

    fn choose_mode(&self, iters: usize) -> SearchMode {
        match iters % 8 {
            0 => SearchMode::SomeProgress((iters / 8) % 32),
            1 => SearchMode::MaxProgress(2),
            2 => SearchMode::LocalMinima,
            3 => SearchMode::SomeProgress(5),
            4 => SearchMode::HalfProgress,

            _ => SearchMode::Standard,
        }
    }

    fn recreate_store(
        &self,
        startctx: &ContextWrapper<T>,
        steps: &[HistoryAlias<T>],
        mode: SearchMode,
    ) -> anyhow::Result<()> {
        let mut ctx = startctx.clone();
        let mut iter = steps.iter().peekable();
        while let Some(hist) = iter.next() {
            // It doesn't actually matter what the last one is, so we skip it.
            if iter.peek().is_none() {
                break;
            }
            if self.queue.db().remember_processed(ctx.get()).unwrap() {
                ctx.replay(self.world, *hist);
                ctx.remove_history();
            } else {
                let prev = Some(ctx.get().clone());
                let elapsed = ctx.elapsed();
                let next = self.recreate_step(ctx);
                if let Some((ci, _)) = next
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.recent_history().last() == Some(hist))
                    .min_by_key(|(_, c)| c.elapsed())
                {
                    // Assumption: no subsequent state leads to victory (aside from the last state?)
                    ctx = next[ci].clone();
                    self.queue.extend_keep_one(next, &ctx, &prev)?;
                    ctx.remove_history();
                } else {
                    // We didn't find the desired state.
                    // Check whether this is a no-op. If so, we can skip pushing states into the queue,
                    // since next iteration will regenerate them.
                    ctx = ContextWrapper::with_elapsed(prev.unwrap(), elapsed);
                    if ctx.can_replay(self.world, *hist) {
                        let c = ctx.get().clone();
                        ctx.replay(self.world, *hist);
                        ctx.remove_history();
                        if ctx.get() == &c {
                            continue;
                        }
                    }
                    // Otherwise, something went wrong.
                    panic!("Failed to recreate \"{}\" at {:?}\n{:?}", hist, ctx, steps);
                }
            }
        }
        let prev = Some(ctx.get().clone());
        let next = self.recreate_step(ctx);
        let next = self.extract_solutions(next, &prev, mode);
        self.queue.extend(next, &prev)
    }

    pub fn search(self) -> Result<(), std::io::Error> {
        let finished = AtomicBool::new(false);
        let workers_done = AtomicUsize::new(0);
        let start = Mutex::new(Instant::now());
        let num_threads = rayon::current_num_threads();
        let num_workers = (num_cpus::get() * 2 + 1) / 3;
        let res = Mutex::new(Ok(()));
        log::info!(
            "Starting search with {} workers ({} threads)",
            num_workers, num_threads
        );

        struct AtExit<'a> {
            flag: &'a AtomicBool,
        }
        impl<'a> Drop for AtExit<'a> {
            fn drop(&mut self) {
                self.flag.store(true, Ordering::Release);
            }
        }

        let run_worker = |i| {
            let mode = mode_by_index(i);
            let mut done = false;
            let mut no_progress = 0;

            // Enforce all workers exiting immediately upon one worker exiting (e.g. panic/assert)
            let _at_exit = AtExit { flag: &finished };

            while !finished.load(Ordering::Acquire)
                && workers_done.load(Ordering::Acquire) < num_workers
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
                    SearchMode::MaxProgress(n) => self.queue.pop_max_progress(n),
                    SearchMode::HalfProgress => self.queue.pop_half_progress(2),
                    SearchMode::SomeProgress(p) => self.queue.pop_min_progress(p, 2),
                    SearchMode::LocalMinima => self.queue.pop_local_minima(),
                    SearchMode::Greedy => self
                        .queue
                        .pop_round_robin(self.organic_level.load(Ordering::Acquire) / 2),
                    _ => self.queue.pop_round_robin(0),
                };
                match items {
                    Ok(items) => {
                        if items.is_empty() {
                            if !done {
                                done = true;
                                workers_done.fetch_add(1, Ordering::Release);
                            }
                            sleep(Duration::from_secs(1));
                            continue;
                        }

                        if done {
                            workers_done.fetch_sub(1, Ordering::Acquire);
                            done = false;
                        }

                        self.held.fetch_add(items.len(), Ordering::Release);

                        if current_mode == SearchMode::Greedy {
                            for ctx in items {
                                self.held.fetch_sub(1, Ordering::Release);
                                if self.queue.db().remember_processed(ctx.get()).unwrap() {
                                    continue;
                                }
                                let iters = self.iters.fetch_add(1, Ordering::AcqRel) + 1;
                                self.check_status_update(&start, iters, &ctx);
                                let progress = self.queue.db().progress(ctx.get());

                                // get remaining locations
                                let remaining =
                                    self.queue.db().scorer().remaining_locations(ctx.get());

                                let max_time = self.queue.max_time();
                                let mut psearch = ctx.clone();

                                for loc_id in &remaining {
                                    psearch.get_mut().skip(*loc_id);
                                }

                                let results: Vec<_> = remaining
                                    .into_par_iter()
                                    .map(|loc_id| {
                                        let mut c = psearch.clone();
                                        c.get_mut().reset(loc_id);

                                        match greedy_search(self.world, &c, max_time, 2) {
                                            Ok(c) | Err(c) => c,
                                        }
                                    })
                                    .collect();

                                self.greedies.fetch_add(1, Ordering::Release);
                                if !results.is_empty()
                                    && self.organic_level.load(Ordering::Acquire) == progress
                                {
                                    self.organic_level
                                        .fetch_max(progress + 1, Ordering::Release);
                                }

                                for mut c in results {
                                    let hist = c.remove_history().0;
                                    if hist.is_empty() {
                                        continue;
                                    }

                                    if let Err(e) =
                                        self.recreate_store(&ctx, &hist, SearchMode::Greedy)
                                    {
                                        log::error!("Thread greedy exiting due to error: {:?}", e);
                                        let mut r = res.lock().unwrap();
                                        if r.is_ok() {
                                            *r = Err(e);
                                            finished.store(true, Ordering::Release);
                                        }
                                        return;
                                    }
                                }
                            }
                        } else {
                            let results: Vec<_> = items
                                .into_par_iter()
                                .filter_map(|ctx| {
                                    self.held.fetch_sub(1, Ordering::Release);
                                    if self.queue.db().remember_processed(ctx.get()).unwrap() {
                                        return None;
                                    }
                                    let iters = self.iters.fetch_add(1, Ordering::AcqRel) + 1;
                                    let progress = self.queue.db().progress(ctx.get());
                                    let prev = Some(ctx.get().clone());
                                    let vec = self.process_one(ctx, iters, &start);
                                    if progress == self.organic_level.load(Ordering::Acquire)
                                        && vec.iter().any(|c| {
                                            self.queue.db().progress(c.get()) == progress + 1
                                        })
                                    {
                                        self.organic_level
                                            .fetch_max(progress + 1, Ordering::Release);
                                    }
                                    Some((prev, vec))
                                })
                                .collect();
                            if results.is_empty() {
                                no_progress += 1;
                                if !done {
                                    done = true;
                                    workers_done.fetch_add(1, Ordering::Release);
                                }
                                sleep(Duration::from_secs(no_progress));
                                continue;
                            } else {
                                no_progress = 0;
                            }
                            if let Err(e) = self.queue.extend_groups(results.into_iter().map(
                                |(prev, nexts)| (self.extract_solutions(nexts, &prev, mode), prev),
                            )) {
                                let mut r = res.lock().unwrap();
                                log::error!("Thread {} exiting due to error: {:?}", i, e);
                                if r.is_ok() {
                                    *r = Err(e);
                                    finished.store(true, Ordering::Release);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Thread {} exiting due to error: {:?}", i, e);
                        let mut r = res.lock().unwrap();
                        if r.is_ok() {
                            *r = Err(e);
                            finished.store(true, Ordering::Release);
                        }
                        return;
                    }
                };
            }
            log::info!(
                "Thread {} exiting: fin={} done={}",
                i,
                finished.load(Ordering::Acquire),
                workers_done.load(Ordering::Acquire)
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
                for i in 0..num_workers {
                    sc2.spawn(move |_| run_worker(i));
                }
            });

            log::info!("Workers all exited, marking finished");
            finished.store(true, Ordering::Release);
        });
        let (iskips, pskips, dskips, dpskips) = self.queue.skip_stats();
        log::info!(
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
        self.solutions.lock().unwrap().export()
    }

    fn process_one(
        &self,
        mut ctx: ContextWrapper<T>,
        iters: usize,
        start: &Mutex<Instant>,
    ) -> Vec<ContextWrapper<T>> {
        self.check_status_update(start, iters, &ctx);

        if ctx.get().count_visits() + ctx.get().count_skips() >= W::NUM_LOCATIONS {
            if self.world.won(ctx.get()) {
                self.handle_solution(&mut ctx, &None, SearchMode::Unknown);
            } else {
                self.deadends.fetch_add(1, Ordering::Release);
            }
            return Vec::new();
        }

        self.single_step(ctx)
    }

    fn check_status_update(&self, start: &Mutex<Instant>, iters: usize, ctx: &ContextWrapper<T>) {
        if iters % 100_000 == 0 {
            self.print_status_update(start, iters, 100_000, ctx);
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
        log::debug!("{} iters took {:?}", num_rounds, s.elapsed());
        *s = Instant::now();

        let sols = self.solutions.lock().unwrap();
        if iters > 10_000_000 && sols.unique() > 4 {
            self.queue.set_max_time(sols.best());
        }
        if iters == 100_000 || iters % 1_000_000 == 0 {
            self.queue.print_queue_histogram();
        }
        let (iskips, pskips, dskips, dpskips) = self.queue.skip_stats();
        let max_time = self.queue.max_time();
        let pending = self.held.load(Ordering::Acquire);
        println!(
            "--- Round {} (solutions={}, unique={}, dead-ends={}, limit={}ms, best={}ms, greedy={}, org={}) ---\n\
            Stats: heap={}; pending={}; db={}; total={}; seen={}; proc={};\n\
            estimates={}; cached={}; evictions={}; retrievals={}\n\
            skips: push:{} time, {} dups; pop: {} time, {} dups; bgdel={}\n\
            heap min: {}\n\
            db bests: {}\n\
            {}",
            iters,
            sols.len(),
            sols.unique(),
            self.deadends.load(Ordering::Acquire),
            max_time,
            sols.best(),
            self.greedies.load(Ordering::Acquire),
            self.organic_level.load(Ordering::Acquire),
            self.queue.heap_len(),
            pending,
            self.queue.db_len(),
            pending + self.queue.len(),
            self.queue.seen(),
            self.queue.db().processed(),
            self.queue.estimates(),
            self.queue.cached_estimates(),
            self.queue.evictions(),
            self.queue.retrievals(),
            iskips,
            dskips,
            pskips,
            dpskips,
            self.queue.background_deletes(),
            self.queue
                .heap_bests()
                .into_iter()
                .map(|n| match n {
                    Some(n) => n.to_string(),
                    None => String::from("-"),
                })
                .collect::<Vec<_>>()
                .join(", "),
            self.queue
                .db()
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
