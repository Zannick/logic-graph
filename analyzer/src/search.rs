use crate::access::*;
use crate::context::*;
use crate::estimates;
use crate::heap::RocksBackedQueue;
use crate::matchertrie::*;
use crate::minimize::*;
use crate::observer::{record_observations, Observer};
use crate::solutions::{Solution, SolutionCollector, SolutionResult};
use crate::world::*;
use anyhow::Result;
use log;
use rayon::prelude::*;
use std::fmt::Debug;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
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
    GreedyMax,
    Start,
    Minimized,
    Mode(usize),
    MutateSpots,
    MutateCollections,
    Unknown,
    Similar,
}

fn mode_by_index(index: usize) -> SearchMode {
    match index % 16 {
        1 => SearchMode::Greedy,
        6 | 10 | 14 => SearchMode::Dependent,
        5 => SearchMode::MaxProgress(4),
        11 => SearchMode::SomeProgress(3),
        2 | 3 | 13 => SearchMode::LocalMinima,
        15 => SearchMode::Mode(8),
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
    let spot_map = accessible_spots(world, ctx, max_time, false);
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
    let mut result = Vec::new();
    for loc in world.get_spot_locations(ctx.get().position()) {
        if ctx.get().todo(loc) && loc.can_access(ctx.get(), world) {
            // Get the item and mark the location visited.
            // If it's a hybrid, also move along the exit.
            let mut newctx = ctx.clone();
            newctx.visit_maybe_exit(world, loc);
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
        if act.can_access(ctx.get(), world) {
            let mut c2 = ctx.clone();
            c2.activate(world, act);
            if c2.get() != ctx.get() {
                result.push(c2);
            }
        }
    }
    for act in world.get_spot_actions(ctx.get().position()) {
        if act.can_access(ctx.get(), world) {
            let mut c2 = ctx.clone();
            c2.activate(world, act);
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
    let movement_state = ctx.get().get_movement_state(world);
    let mut results = Vec::new();
    for ce in world.get_condensed_edges_from(ctx.get().position()) {
        if ce.time(world, ctx.get()) + ctx.elapsed() <= max_time
            && ce.can_access(world, ctx.get(), movement_state)
        {
            let mut newctx = ctx.clone();
            newctx.move_condensed_edge(world, ce);
            results.push(newctx);
        }
    }
    for exit in world.get_spot_exits(ctx.get().position()) {
        if exit.time(ctx.get(), world) + ctx.elapsed() <= max_time
            && exit.can_access(ctx.get(), world)
        {
            let mut newctx = ctx.clone();
            newctx.exit(world, exit);
            results.push(newctx);
        }
    }
    for warp in world.get_warps() {
        if warp.time(ctx.get(), world) + ctx.elapsed() <= max_time
            && warp.can_access(ctx.get(), world)
        {
            let mut newctx = ctx.clone();
            newctx.warp(world, warp);
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
    let movement_state = ctx.get().get_movement_state(world);
    let mut results = Vec::new();
    for &dest in world.get_area_spots(ctx.get().position()) {
        let ltt = ctx.get().local_travel_time(movement_state, dest);
        if ltt < u32::MAX && ltt + ctx.elapsed() <= max_time {
            let mut newctx = ctx.clone();
            newctx.move_local(world, dest, ltt);
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
    solve_trie: Arc<MatcherTrie<<T::Observer as Observer>::Matcher>>,
    solutions: Arc<Mutex<SolutionCollector<T>>>,
    queue: RocksBackedQueue<'a, W, T>,
    solution_cvar: Condvar,
    iters: AtomicUsize,
    deadends: AtomicU32,
    greedies: AtomicUsize,
    greedy_in_comm: AtomicUsize,
    greedy_out_comm: AtomicUsize,
    held: AtomicUsize,
    last_clean: AtomicUsize,
    solves_since_clean: AtomicUsize,
    last_solve: AtomicUsize,
    next_threshold: AtomicUsize,
    organic_solution: AtomicBool,
    any_solution: AtomicBool,
    organic_level: AtomicUsize,
    mutated: AtomicUsize,
    finished: AtomicBool,
}

impl<'a, W, T, L, E> Search<'a, W, T>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<Context = T, ExitId = L::ExitId, LocId = L::LocId, Currency = L::Currency>,
{
    pub fn new<P>(
        world: &'a W,
        ctx: T,
        routes: Vec<ContextWrapper<T>>,
        db_path: P,
    ) -> Result<Search<'a, W, T>, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let solve_trie: Arc<MatcherTrie<<T::Observer as Observer>::Matcher>> = Arc::default();
        let mut solutions = SolutionCollector::<T>::new(
            "data/solutions.txt",
            "data/previews.txt",
            "data/best.txt",
            &ctx,
        )?;

        let startctx = ContextWrapper::new(ctx);
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
                "Provided extra routes: {} winners, {} not; winning times: {:?}",
                wins.len(),
                others.len(),
                wins.iter().map(|c| c.elapsed()).collect::<Vec<_>>()
            );
        } else if !others.is_empty() {
            log::info!("Provided {} non-winning routes.", others.len());
        } else {
            log::info!("No routes provided, starting from scratch.");
        }

        let initial_max_time = if let Some(wonctx) = wins.last() {
            let max_time = wonctx.elapsed();
            let sol = wonctx.to_solution();
            if solutions.insert_solution(sol.clone(), world).accepted() {
                record_observations(startctx.get(), world, sol, 1, &solve_trie);
            }
            for w in &wins {
                let sol = w.to_solution();

                // Insert the solution.
                if solutions.insert_solution(sol.clone(), world).accepted() {
                    record_observations(startctx.get(), world, sol.clone(), 1, &solve_trie);
                }
                // Try trie-minimization; if successful, insert and record that solution.
                if let Some(solution) =
                    trie_minimize(world, startctx.get(), sol.clone(), &solve_trie)
                        .map(|c| c.into_solution())
                {
                    if solutions
                        .insert_solution(solution.clone(), world)
                        .accepted()
                    {
                        record_observations(
                            startctx.get(),
                            world,
                            solution.clone(),
                            1,
                            &solve_trie,
                        );
                    }
                }
            }
            max_time + max_time / 128
        } else {
            estimates::UNREASONABLE_TIME
        };
        log::info!(
            "Minimization results: {} solutions, {} others",
            solutions.len(),
            others.len()
        );

        let solutions = Arc::new(Mutex::new(solutions));

        let queue = RocksBackedQueue::new(
            db_path,
            world,
            &startctx,
            initial_max_time,
            2_097_152,
            262_144,
            524_288,
            4_096,
            16_384,
            solutions.clone(),
        )
        .unwrap();
        queue.push(startctx.clone(), &None).unwrap();
        log::info!("Max time to consider is now: {}ms", queue.max_time());
        let s = Search {
            world,
            startctx,
            solve_trie,
            solutions,
            queue,
            solution_cvar: Condvar::new(),
            iters: 0.into(),
            deadends: 0.into(),
            held: 0.into(),
            greedies: 0.into(),
            greedy_in_comm: 0.into(),
            greedy_out_comm: 0.into(),
            last_clean: 0.into(),
            solves_since_clean: 0.into(),
            last_solve: 0.into(),
            next_threshold: 0.into(),
            organic_solution: false.into(),
            any_solution: AtomicBool::new(!wins.is_empty()),
            organic_level: 0.into(),
            mutated: 0.into(),
            finished: false.into(),
        };
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

    fn clean_solutions(&self) {
        let mut sols = self.solutions.lock().unwrap();
        let min_visits = self.min_progress();
        let orig = sols.len();
        let start = Instant::now();
        sols.clean();
        log::info!("Cleaned out solutions in {:?}", start.elapsed());
        if sols.len() < orig {
            let start = Instant::now();
            self.solve_trie.clear();
            for solution in sols.iter() {
                record_observations(
                    self.startctx.get(),
                    self.world,
                    solution,
                    // normalize total visits to required visits
                    min_visits,
                    &self.solve_trie,
                );
            }
            log::info!("Reset solve trie in {:?}", start.elapsed());
        }
    }

    fn min_progress(&self) -> usize {
        if let Some(qmp) = self.queue.min_progress() {
            if let Some(dbmp) = self.queue.db().min_progress() {
                std::cmp::min(qmp, dbmp)
            } else {
                qmp
            }
        } else if let Some(dbmp) = self.queue.db().min_progress() {
            dbmp
        } else {
            1
        }
    }

    fn handle_solution(&self, ctx: &mut ContextWrapper<T>, prev: &Option<T>, mode: SearchMode) {
        // If prev is None we don't know the prev state
        // nor whether we have recent history in ctx--either we got this state from the queue and it has none
        // or it was recreated and stored in the db, in which case we can get it from the db as well.
        // But if prev is true, we must only record the state, since
        // recording `next` requires all the states at once.
        if prev.is_some() {
            self.queue.db().record_one(ctx, prev, true).unwrap();
        }

        if !matches!(mode, SearchMode::Similar | SearchMode::Minimized) {
            self.organic_solution.store(true, Ordering::Release);
        }
        self.any_solution.store(true, Ordering::Release);

        let mut old_time = self.queue.max_time();
        let iters = self.iters.load(Ordering::Acquire);

        let history = self.queue.db().get_history(ctx.get()).unwrap();
        let (elapsed, ..) = self.queue.db().get_best_times(ctx.get()).unwrap();

        let solution = Arc::new(Solution { elapsed, history });

        let mut sols = self.solutions.lock().unwrap();
        if iters > 10_000_000 || sols.unique() > 1_000 {
            self.queue.set_max_time(elapsed);
        } else if iters > 5_000_000 || sols.unique() > 100 {
            self.queue.set_max_time(elapsed + elapsed / 1000);
        } else if iters > 2_000_000 && sols.unique() > 4 {
            self.queue.set_max_time(elapsed + elapsed / 100);
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

        let min_progress = self.min_progress();
        let res = sols.insert_solution(solution.clone(), self.world);
        if res.accepted() {
            self.solves_since_clean.fetch_add(1, Ordering::Release);
            self.last_solve
                .fetch_max(self.iters.load(Ordering::Acquire), Ordering::Release);
            self.solution_cvar.notify_one();
        }
        drop(sols); // release before recording observations/minimizing
        if res.accepted() {
            if res == SolutionResult::IsUnique {
                log::debug!(
                    "Recording new unique solution from {:?} mode: {}ms",
                    mode,
                    elapsed
                );
            } else {
                log::debug!("Recording solution from {:?} mode: {}ms", mode, elapsed);
            }
            record_observations(
                self.startctx.get(),
                self.world,
                solution.clone(),
                min_progress,
                &self.solve_trie,
            );
        }

        let min_ctx = match mode {
            SearchMode::Similar => {
                pinpoint_minimize(self.world, self.startctx.get(), solution.clone())
            }
            SearchMode::Minimized => None,
            _ => trie_minimize(
                self.world,
                self.startctx.get(),
                solution.clone(),
                &self.solve_trie,
            ),
        };

        let mut sols = self.solutions.lock().unwrap();
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

            let solution = ctx.into_solution();
            let res = sols.insert_solution(solution.clone(), self.world);
            drop(sols);
            if res.accepted() {
                self.solves_since_clean.fetch_add(1, Ordering::Release);
                self.last_solve
                    .fetch_max(self.iters.load(Ordering::Acquire), Ordering::Release);
                self.solution_cvar.notify_one();
                record_observations(
                    self.startctx.get(),
                    self.world,
                    solution.clone(),
                    min_progress,
                    &self.solve_trie,
                );
            }

            self.recreate_store(&self.startctx, &solution.history, SearchMode::Minimized)
                .unwrap();
        }
    }

    fn extract_solutions(
        &self,
        states: Vec<ContextWrapper<T>>,
    ) -> (Vec<ContextWrapper<T>>, Vec<ContextWrapper<T>>) {
        let max_time = self.queue.max_time();
        let mut solutions = Vec::new();
        (
            states
                .into_iter()
                .filter_map(|ctx| {
                    if ctx.elapsed() > max_time {
                        None
                    } else if self.world.won(ctx.get()) {
                        solutions.push(ctx);
                        None
                    } else {
                        Some(ctx)
                    }
                })
                .collect(),
            solutions,
        )
    }

    fn extend_and_handle_solutions(
        &self,
        states: Vec<ContextWrapper<T>>,
        prev: &Option<T>,
        mode: SearchMode,
    ) -> anyhow::Result<()> {
        let (next, solutions) = self.extract_solutions(states);
        rayon::join(
            move || self.queue.extend(next, prev),
            move || {
                for mut ctx in solutions {
                    // The state is added to the db in handle_solution
                    // and the ctx no longer has history attached.
                    self.handle_solution(&mut ctx, prev, mode)
                }
            },
        )
        .0
    }

    fn single_step(&self, ctx: ContextWrapper<T>) -> Vec<ContextWrapper<T>> {
        single_step(self.world, ctx, self.queue.max_time())
    }

    fn recreate_step(&self, ctx: ContextWrapper<T>) -> Vec<ContextWrapper<T>> {
        single_step_with_local(self.world, ctx, u32::MAX)
    }

    fn choose_mode(&self, iters: usize) -> SearchMode {
        if !self.any_solution.load(Ordering::Acquire) {
            SearchMode::MaxProgress(4)
        } else {
            match iters % 8 {
                0 => SearchMode::SomeProgress((iters / 8) % 32),
                1 => SearchMode::MaxProgress(2),
                2 => SearchMode::LocalMinima,
                3 => SearchMode::SomeProgress(5),
                4 => SearchMode::HalfProgress,
                5 => SearchMode::Mode(4),

                _ => SearchMode::Standard,
            }
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
                ctx.assert_and_replay(self.world, *hist);
                ctx.remove_history();
            } else {
                let prev = Some(ctx.get().clone());
                let elapsed = ctx.elapsed();
                let time_since_visit = ctx.time_since_visit();
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
                    ctx = ContextWrapper::with_times(prev.unwrap(), elapsed, time_since_visit);
                    if ctx.can_replay(self.world, *hist) {
                        let c = ctx.get().clone();
                        ctx.replay(self.world, *hist);
                        ctx.remove_history();
                        if ctx.get() == &c {
                            continue;
                        }
                    }
                    // Otherwise, something went wrong.
                    panic!(
                        "Failed to recreate \"{}\":\n{}\nat {:?}\n{:?}\nOptions were: {:?}",
                        hist,
                        ctx.explain_pre_replay(self.world, *hist),
                        ctx,
                        steps,
                        next.iter()
                            .map(|c| c.recent_history().last())
                            .collect::<Vec<_>>()
                    );
                }
            }
        }
        let prev = Some(ctx.get().clone());
        let next = self.recreate_step(ctx);
        self.extend_and_handle_solutions(next, &prev, mode)
    }

    pub fn search(self) -> Result<(), std::io::Error> {
        let workers_done = AtomicUsize::new(0);
        let start = Mutex::new(Instant::now());
        let num_threads = rayon::current_num_threads();
        let num_workers = (num_cpus::get() * 2 + 1) / 3;
        let res = Mutex::new(Ok(()));
        log::info!(
            "Starting search with {} workers ({} threads)",
            num_workers,
            num_threads
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
            let _at_exit = AtExit {
                flag: &self.finished,
            };

            let greedy_error = |e| {
                log::error!("Thread greedy exiting due to error: {:?}", e);
                let mut r = res.lock().unwrap();
                if r.is_ok() {
                    *r = Err(e);
                    self.finished.store(true, Ordering::Release);
                }
            };
            let incr_organic = |progress| {
                let org = self.organic_level.load(Ordering::Acquire);
                if org == progress || Some(org) <= self.queue.min_progress() {
                    self.organic_level
                        .fetch_max(progress + 1, Ordering::Release);
                }
            };

            while !self.finished.load(Ordering::Acquire)
                && workers_done.load(Ordering::Acquire) < num_workers
            {
                let iters = self.iters.load(Ordering::Acquire);
                let current_mode = if iters < 500_000 {
                    SearchMode::Standard
                } else if mode == SearchMode::Dependent {
                    self.choose_mode(iters)
                } else if i % 2 == 0 && !self.any_solution.load(Ordering::Acquire) {
                    SearchMode::GreedyMax
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
                    SearchMode::GreedyMax => self.queue.pop_max_progress(1),
                    SearchMode::Mode(n) => self.queue.pop_mode(n),
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

                        // This is probably where we lookup in the solve trie and attempt to recreate if we find something.

                        if current_mode == SearchMode::Greedy
                            || current_mode == SearchMode::GreedyMax
                        {
                            items.into_par_iter().for_each(|mut ctx| {
                                self.held.fetch_sub(1, Ordering::Release);
                                if self.queue.db().remember_processed(ctx.get()).unwrap() {
                                    return;
                                }
                                let iters = self.iters.fetch_add(1, Ordering::AcqRel) + 1;
                                self.check_status_update(&start, iters, &ctx);
                                let progress = ctx.get().count_visits();

                                // get remaining locations
                                let remaining: Vec<_> = self
                                    .world
                                    .get_all_locations()
                                    .into_iter()
                                    .filter(|loc| ctx.get().todo(loc))
                                    .collect();
                                if remaining.is_empty() {
                                    if self.world.won(ctx.get()) {
                                        self.handle_solution(&mut ctx, &None, SearchMode::Unknown);
                                    } else {
                                        self.deadends.fetch_add(1, Ordering::Release);
                                    }
                                    return;
                                }
                                let comm = W::location_community(
                                    nearest_location_by_heuristic(
                                        self.world,
                                        ctx.get(),
                                        remaining.iter().copied(),
                                        self.queue.db().scorer().get_algo(),
                                    )
                                    .unwrap()
                                    .id(),
                                );
                                let (in_comm, out_comm) =
                                    remaining.into_iter().partition::<Vec<_>, _>(|loc| {
                                        W::location_community(loc.id()) == comm
                                    });

                                let max_time = self.queue.max_time();

                                rayon::join(
                                    || {
                                        let ct = in_comm.len();
                                        in_comm.into_par_iter().for_each(|loc| {
                                            match self.process_one_greedy(
                                                &ctx,
                                                loc,
                                                max_time,
                                                current_mode,
                                            ) {
                                                Ok(true) => incr_organic(progress),
                                                Err(e) => greedy_error(e),
                                                _ => (),
                                            }
                                        });
                                        self.greedy_in_comm.fetch_add(ct, Ordering::AcqRel);
                                    },
                                    || {
                                        let ct = out_comm
                                            .into_par_iter()
                                            .filter_map(|loc| {
                                                match self.process_one_greedy(
                                                    &ctx,
                                                    loc,
                                                    max_time,
                                                    current_mode,
                                                ) {
                                                    Ok(true) => Some(incr_organic(progress)),
                                                    Err(e) => {
                                                        greedy_error(e);
                                                        None
                                                    }
                                                    _ => None,
                                                }
                                            })
                                            .take_any(2)
                                            .count();
                                        self.greedy_out_comm.fetch_add(ct, Ordering::AcqRel);
                                    },
                                );

                                self.greedies.fetch_add(1, Ordering::Release);
                            });
                        } else {
                            let results: Vec<_> = items
                                .into_par_iter()
                                .filter_map(|ctx| {
                                    self.held.fetch_sub(1, Ordering::Release);
                                    if self.queue.db().remember_processed(ctx.get()).unwrap() {
                                        return None;
                                    }
                                    let iters = self.iters.fetch_add(1, Ordering::AcqRel) + 1;
                                    let visits = ctx.get().count_visits();
                                    let prev = Some(ctx.get().clone());
                                    if let Some(vec) = self.process_one(ctx, iters, &start) {
                                        if visits == self.organic_level.load(Ordering::Acquire)
                                            && vec
                                                .iter()
                                                .any(|c| c.get().count_visits() == visits + 1)
                                        {
                                            self.organic_level
                                                .fetch_max(visits + 1, Ordering::Release);
                                        }

                                        if let Err(e) =
                                            self.extend_and_handle_solutions(vec, &prev, mode)
                                        {
                                            let mut r = res.lock().unwrap();
                                            log::error!(
                                                "Thread {} exiting due to error: {:?}",
                                                i,
                                                e
                                            );
                                            if r.is_ok() {
                                                *r = Err(e);
                                                self.finished.store(true, Ordering::Release);
                                            }
                                            None
                                        } else {
                                            Some(())
                                        }
                                    } else {
                                        None
                                    }
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
                            }

                            no_progress = 0;
                        }
                    }
                    Err(e) => {
                        log::error!("Thread {} exiting due to error: {:?}", i, e);
                        let mut r = res.lock().unwrap();
                        if r.is_ok() {
                            *r = Err(e);
                            self.finished.store(true, Ordering::Release);
                        }
                        return;
                    }
                };
            }
            log::trace!(
                "Thread {} exiting: fin={} done={}",
                i,
                self.finished.load(Ordering::Acquire),
                workers_done.load(Ordering::Acquire)
            );
        };

        rayon::scope(|scope| {
            // Background db cleanup thread
            scope.spawn(|_| {
                let sleep_time = Duration::from_secs(10);
                while !self.finished.load(Ordering::Acquire) {
                    let len = self.queue.db_len();
                    if len < 1_000_000 {
                        sleep(sleep_time);
                        continue;
                    }
                    self.queue.db_cleanup(65_536, &self.finished).unwrap();
                }
            });

            // Background solution mutator.
            scope.spawn(|_| {
                let max_wait_time = Duration::from_secs(300);
                let mut sols = self.solutions.lock().unwrap();
                while !self.finished.load(Ordering::Acquire) {
                    while let Some(sol) = sols.next_unprocessed() {
                        if sol.elapsed > self.queue.max_time() {
                            // We should have dropped these from the collector already.
                            continue;
                        }
                        drop(sols);
                        let revisits =
                            mutate_spot_revisits(self.world, self.startctx.get(), sol.clone());
                        for revisit in revisits {
                            if revisit.elapsed() < sol.elapsed {
                                self.recreate_store(
                                    &self.startctx,
                                    revisit.recent_history(),
                                    SearchMode::MutateSpots,
                                )
                                .unwrap();
                            }
                        }
                        if let Some(reordered) = mutate_collection_steps(
                            self.world,
                            self.startctx.get(),
                            self.queue.max_time(),
                            4,
                            4_096,
                            sol,
                            self.queue.db().scorer().get_algo(),
                        ) {
                            self.recreate_store(
                                &self.startctx,
                                reordered.recent_history(),
                                SearchMode::MutateCollections,
                            )
                            .unwrap();
                        }
                        self.mutated.fetch_add(1, Ordering::Release);
                        sols = self.solutions.lock().unwrap();
                        if self.finished.load(Ordering::Acquire) {
                            return;
                        }
                    }
                    (sols, _) = self.solution_cvar.wait_timeout(sols, max_wait_time).unwrap();
                    if self.finished.load(Ordering::Acquire) {
                        return;
                    }
                }
            });

            rayon::scope(|sc2| {
                for i in 0..num_workers {
                    sc2.spawn(move |_| run_worker(i));
                }
            });

            log::debug!("Workers all exited, marking finished");
            self.finished.store(true, Ordering::Release);
            self.solution_cvar.notify_all();
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

    fn process_one_greedy(
        &self,
        ctx: &ContextWrapper<T>,
        loc: &L,
        max_time: u32,
        current_mode: SearchMode,
    ) -> anyhow::Result<bool> {
        let res = access_location_after_actions(
            self.world,
            ctx.clone(),
            loc.id(),
            max_time,
            if current_mode == SearchMode::GreedyMax {
                9
            } else {
                4
            },
            4_096,
            self.queue.db().scorer().get_algo(),
        );
        let found = res.is_ok();
        match res {
            Ok(mut c) => {
                let hist = c.remove_history().0;
                if !hist.is_empty() {
                    self.recreate_store(&ctx, &hist, current_mode).map(|_| true)
                } else {
                    Ok(found)
                }
            }
            _ => Ok(false),
        }
    }

    fn process_one(
        &self,
        mut ctx: ContextWrapper<T>,
        iters: usize,
        start: &Mutex<Instant>,
    ) -> Option<Vec<ContextWrapper<T>>> {
        self.check_status_update(start, iters, &ctx);

        if ctx.get().count_visits() >= W::NUM_CANON_LOCATIONS {
            if self.world.won(ctx.get()) {
                self.handle_solution(&mut ctx, &None, SearchMode::Unknown);
            } else {
                self.deadends.fetch_add(1, Ordering::Release);
            }
            return Some(Vec::new());
        }

        if let Some(win) = trie_search(self.world, &ctx, self.queue.max_time(), &self.solve_trie) {
            // Handles recording the solution as well.
            self.recreate_store(&ctx, win.recent_history(), SearchMode::Similar)
                .unwrap();
            None
        } else {
            Some(self.single_step(ctx))
        }
    }

    fn check_status_update(&self, start: &Mutex<Instant>, iters: usize, ctx: &ContextWrapper<T>) {
        let last_solve = self.last_solve.load(Ordering::Acquire);
        static PREVIEWS_RATE: usize = 4_096;
        if iters % PREVIEWS_RATE == 0 && last_solve + PREVIEWS_RATE <= iters {
            let mut sols = self.solutions.lock().unwrap();
            // Recheck after the lock is acquired.
            let last_solve = self.last_solve.load(Ordering::Acquire);
            if last_solve + PREVIEWS_RATE <= iters {
                sols.write_previews_if_pending().unwrap();
            }
        }
        if iters % 100_000 == 0 {
            self.print_status_update(start, iters, 100_000, ctx);

            if iters % 1_000_000 == 0 {
                let last_clean = self.last_clean.load(Ordering::Acquire);
                let solves_since = self.solves_since_clean.load(Ordering::Acquire);
                if solves_since > 0 {
                    if last_clean + 10_000_000 <= iters {
                        if solves_since > 20 || last_solve + 20_000_000 <= iters {
                            self.solves_since_clean.store(0, Ordering::Release);
                            self.last_clean.store(iters, Ordering::Release);
                            self.clean_solutions();
                            self.next_threshold.store(
                                std::cmp::max(iters + 50_000_000, iters * 2),
                                Ordering::Release,
                            );
                        }
                    }
                } else if last_clean > 0 && self.next_threshold.load(Ordering::Acquire) <= iters {
                    // Minimum 250M after the last clean
                    if (iters - last_clean)
                        // threshold step
                        / (std::cmp::max(last_clean + 50_000_000, last_clean * 2) - last_clean)
                        >= 5
                    {
                        log::info!(
                            "No solves in {} attempts since last clean ({}x), giving up.",
                            iters - last_clean,
                            iters / last_clean,
                        );
                        self.finished.store(true, Ordering::Release);
                    } else {
                        log::info!(
                            "No solves in {} attempts since last clean ({}x)",
                            iters - last_clean,
                            iters / last_clean,
                        );
                    }
                    self.next_threshold.fetch_add(last_clean, Ordering::Release);
                }
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
        let dur = s.elapsed();
        log::debug!(
            "{} iters took {:?}: throughput = {}/sec",
            num_rounds,
            dur,
            num_rounds as f32 / dur.as_secs_f32()
        );
        *s = Instant::now();

        let sols = self.solutions.lock().unwrap();
        let unique = sols.unique();
        let solcount = sols.len();
        let best = sols.best();
        drop(sols);
        if unique > 1_000 || iters > 10_000_000 {
            self.queue.set_max_time(best);
        } else if iters > 5_000_000 || unique > 100 {
            self.queue.set_max_time(best + best / 1_000);
        } else if iters > 2_000_000 && unique > 4 {
            self.queue.set_max_time(best + best / 100);
        }
        if iters == 100_000 || iters % 1_000_000 == 0 {
            self.queue.print_queue_histogram();
        }
        let (iskips, pskips, dskips, dpskips) = self.queue.skip_stats();
        let max_time = self.queue.max_time();
        let pending = self.held.load(Ordering::Acquire);
        // TODO: heap+db range [min bucket, max bucket]
        let heap_bests = self.queue.heap_bests();
        let db_bests = self.queue.db().db_bests();
        let db_best_max = db_bests.iter().rposition(|x| *x != u32::MAX).unwrap_or(0);
        let needed = self.world.items_needed(ctx.get());
        println!(
            "--- Round {} (solutions={}, unique={}, mut={}, limit={}ms, best={}ms) ---\n\
            Stats: heap={}; pending={}; db={}; total={}; seen={}; proc={}; dead-end={}\n\
            trie size={}, depth={}, values={}; estimates={}; cached={}; evictions={}; retrievals={}\n\
            Greedy stats: org level={}, steps done={}, proc_in={}, proc_out={}\n\
            skips: push:{} time, {} dups; pop: {} time, {} dups; bgdel={}\n\
            heap: [{}..={}] mins: {}\n\
            db: [{}..={}] mins: {}\n\
            {}\n\
            Still needs: {:?}",
            iters,
            solcount,
            unique,
            self.mutated.load(Ordering::Acquire),
            max_time,
            best,
            self.queue.heap_len(),
            pending,
            self.queue.db_len(),
            pending + self.queue.len(),
            self.queue.seen(),
            self.queue.db().processed(),
            self.deadends.load(Ordering::Acquire),
            self.solve_trie.size(),
            self.solve_trie.max_depth(),
            self.solve_trie.num_values(),
            self.queue.estimates(),
            self.queue.cached_estimates(),
            self.queue.evictions(),
            self.queue.retrievals(),
            self.organic_level.load(Ordering::Acquire),
            self.greedies.load(Ordering::Acquire),
            self.greedy_in_comm.load(Ordering::Acquire),
            self.greedy_out_comm.load(Ordering::Acquire),
            iskips,
            dskips,
            pskips,
            dpskips,
            self.queue.background_deletes(),
            heap_bests.iter().position(|x| *x != None).unwrap_or(0),
            heap_bests.len().saturating_sub(1),
            heap_bests
                .into_iter()
                .map(|n| match n {
                    Some((n, ..)) => n.to_string(),
                    None => String::from("-"),
                })
                .collect::<Vec<_>>()
                .join(", "),
            db_bests.iter().position(|x| *x != u32::MAX).unwrap_or(0),
            db_best_max,
            db_bests[..=db_best_max]
                .into_iter()
                .map(|n| if *n < u32::MAX {
                    n.to_string()
                } else {
                    String::from("-")
                })
                .collect::<Vec<_>>()
                .join(", "),
            ctx.info(
                self.queue.estimated_remaining_time(ctx),
                self.queue.db().get_last_history_step(ctx).unwrap()
            ),
            if needed.len() > 10 {
                format!("{:?} + {} more types", needed[..10].to_vec(), needed.len() - 10)
            } else {
                format!("{:?}", needed)
            },
        );
    }
}
