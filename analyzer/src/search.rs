use crate::access::*;
use crate::context::*;
use crate::db::RouteDb;
use crate::direct::DirectPathsDb;
use crate::estimates::{ContextScorer, UNREASONABLE_TIME};
use crate::heap::{MetricType, RocksBackedQueue};
use crate::matchertrie::*;
use crate::minimize::*;
use crate::observer::{record_observations, TrieMatcher};
use crate::scoring::ScoreMetric;
use crate::solutions::{Solution, SolutionCollector, SolutionResult, SolutionSuffix};
use crate::world::*;
use anyhow::Result;
use log;
use rayon::prelude::*;
use similar::TextDiff;
use std::fmt::Debug;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

#[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
mod jemalloc {
    use axum::{
        http::StatusCode,
        response::{IntoResponse},
    };

    pub async fn handle_get_heap() -> Result<impl IntoResponse, (StatusCode, String)> {
        let mut prof_ctl = jemalloc_pprof::PROF_CTL.as_ref().unwrap().lock().await;
        require_profiling_activated(&prof_ctl)?;
        let pprof = prof_ctl
            .dump_pprof()
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
        Ok(pprof)
    }

    /// Checks whether jemalloc profiling is activated an returns an error response if not.
    fn require_profiling_activated(
        prof_ctl: &jemalloc_pprof::JemallocProfCtl,
    ) -> Result<(), (StatusCode, String)> {
        if prof_ctl.activated() {
            Ok(())
        } else {
            Err((
                axum::http::StatusCode::FORBIDDEN,
                "heap profiling not activated".into(),
            ))
        }
    }
}

static MAX_DEPTH_FOR_ONE_LOC: usize = 4;
static MAX_GREEDY_DEPTH: usize = 9;
static MAX_STATES_FOR_ONE_LOC: usize = 16_384;

static QUEUE_CAPACITY: usize = 2_097_152;
static QUEUE_MIN_PER_EVICTION: usize = 262_144;
static QUEUE_MAX_PER_EVICTION: usize = 524_288;
static QUEUE_MIN_PER_RESHUFFLE: usize = 4_096;
static QUEUE_MAX_PER_RESHUFFLE: usize = 16_384;

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
    MutateMinimize,
    MutateSpots,
    MutateCollections,
    MutateGreedySteps,
    MutateCanonLocations,
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

pub fn explore<W, T, L>(world: &W, ctx: ContextWrapper<T>, max_time: u32) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    W::Exit: Exit<Context = T, Currency = L::Currency>,
    W::Warp: Warp<Context = T, SpotId = <W::Exit as Exit>::SpotId, Currency = L::Currency>,
{
    let spot_map = accessible_spots(world, ctx, max_time, false);
    let mut vec: Vec<ContextWrapper<T>> = spot_map.into_values().collect();

    vec.par_sort_unstable_by_key(|el| el.elapsed());
    vec
}

pub fn visit_locations<W, T, L>(world: &W, ctx: ContextWrapper<T>) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    W::Exit: Exit<Context = T, Currency = L::Currency>,
{
    let mut result = Vec::new();
    for loc in world.get_spot_locations(ctx.get().position()) {
        if ctx.get().todo(loc) && loc.can_access(ctx.get(), world) {
            // Get the item and mark the location visited.
            // If it's a hybrid, also move along the exit.
            let mut newctx = ctx.clone();
            newctx.visit(world, loc);
            result.push(newctx);
        }
    }
    result
}

pub fn activate_actions<W, T, L>(world: &W, ctx: &ContextWrapper<T>) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    W::Exit: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
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

#[derive(Copy, Clone)]
pub struct SearchOptions {
    pub mutate_max_depth: usize,
    pub mutate_max_states: usize,
    pub local_max_depth: usize,
    pub local_max_states: usize,
    pub greedy_max_depth: usize,
    pub greedy_max_states: usize,
}
impl Default for SearchOptions {
    fn default() -> Self {
        SearchOptions {
            mutate_max_depth: MAX_DEPTH_FOR_ONE_LOC,
            mutate_max_states: MAX_STATES_FOR_ONE_LOC,
            local_max_depth: MAX_DEPTH_FOR_ONE_LOC,
            local_max_states: MAX_STATES_FOR_ONE_LOC,
            greedy_max_depth: MAX_GREEDY_DEPTH,
            greedy_max_states: MAX_STATES_FOR_ONE_LOC,
        }
    }
}

pub struct Search<'a, W, T, TM>
where
    W: World,
    T: Ctx<World = W> + Debug,
    W::Location: Location<Context = T>,
    TM: TrieMatcher<SolutionSuffix<T>, Struct = T>,
{
    world: &'a W,
    startctx: ContextWrapper<T>,
    solve_trie: Arc<MatcherTrie<TM, SolutionSuffix<T>>>,
    solutions: Arc<Mutex<SolutionCollector<T>>>,
    direct_paths: DirectPathsDb<W, T>,
    queue: RocksBackedQueue<'a, W, T>,
    solution_cvar: Condvar,
    options: SearchOptions,

    // stats
    iters: AtomicUsize,
    deadends: AtomicU32,
    greedies: AtomicUsize,
    greedy_misses: AtomicUsize,
    greedy_spots_only: AtomicUsize,
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
    last_organic_improvement: AtomicUsize,
    mutated: AtomicUsize,
    finished: AtomicBool,
}

impl<'a, W, T, TM> Search<'a, W, T, TM>
where
    W: World,
    T: Ctx<World = W> + Debug,
    W::Location: Location<Context = T>,
    W::Exit: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    TM: TrieMatcher<SolutionSuffix<T>, Struct = T>,
{
    pub fn new<P>(
        world: &'a W,
        ctx: T,
        routes: Vec<ContextWrapper<T>>,
        metric: MetricType<'a, W>,
        db_path: P,
        options: SearchOptions,
    ) -> Result<Search<'a, W, T, TM>, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let solve_trie: Arc<MatcherTrie<TM, SolutionSuffix<T>>> = Arc::default();
        let mut solutions = SolutionCollector::<T>::new(
            "data/solutions.txt",
            "data/previews.txt",
            "data/best.txt",
            "data/best-prev.txt",
            &ctx,
        )?;

        let startctx = ContextWrapper::new(ctx);

        let free_sp = ContextScorer::shortest_paths_tree_free_edges(world, startctx.get());

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
                record_observations::<_, _, TM>(startctx.get(), world, sol, 1, &solve_trie);
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
            UNREASONABLE_TIME
        };
        log::info!(
            "Minimization results: {} solutions, {} others",
            solutions.len(),
            others.len()
        );

        let solutions = Arc::new(Mutex::new(solutions));

        let mut vpath = db_path.as_ref().to_owned();
        vpath.push("VERSION");
        let version_diff =
            std::fs::exists(&vpath)? && std::fs::read_to_string(&vpath)? != W::VERSION;
        let delete_dbs = if version_diff {
            print!("Detected db version mismatch. Proceed to delete dbs and start over? (y/N) ");
            std::io::stdout().flush().unwrap();
            let mut str = String::default();
            std::io::stdin().read_line(&mut str).unwrap();
            assert!(str.starts_with(&['y', 'Y']), "Exiting without deleting dbs");
            std::fs::write(&vpath, W::VERSION)?;
            true
        } else {
            false
        };

        let queue = RocksBackedQueue::new(
            db_path.as_ref(),
            world,
            metric,
            initial_max_time,
            QUEUE_CAPACITY,
            QUEUE_MIN_PER_EVICTION,
            QUEUE_MAX_PER_EVICTION,
            QUEUE_MIN_PER_RESHUFFLE,
            QUEUE_MAX_PER_RESHUFFLE,
            delete_dbs,
        )
        .unwrap();
        queue.push(startctx.clone(), &None).unwrap();
        log::info!("Max time to consider is now: {}ms", queue.max_time());

        let mut path = db_path.as_ref().to_owned();
        path.push("routes");
        let (ropts, rcache) = RouteDb::<T>::default_options();
        let route_db = RouteDb::<T>::open(path, ropts, rcache, delete_dbs).unwrap();
        let direct_paths = DirectPathsDb::new(free_sp, route_db);

        let s = Search {
            world,
            startctx,
            solve_trie,
            solutions,
            direct_paths,
            queue,
            solution_cvar: Condvar::new(),
            options,
            iters: 0.into(),
            deadends: 0.into(),
            held: 0.into(),
            greedies: 0.into(),
            greedy_misses: 0.into(),
            greedy_spots_only: 0.into(),
            greedy_in_comm: 0.into(),
            greedy_out_comm: 0.into(),
            last_clean: 0.into(),
            solves_since_clean: 0.into(),
            last_solve: 0.into(),
            next_threshold: 0.into(),
            organic_solution: false.into(),
            any_solution: AtomicBool::new(!wins.is_empty()),
            organic_level: 0.into(),
            last_organic_improvement: 0.into(),
            mutated: 0.into(),
            finished: false.into(),
        };

        log::debug!("Recreating routes...");
        wins.extend(others);
        wins.into_par_iter().for_each(|w| {
            s.recreate_store(&s.startctx, w.recent_history(), SearchMode::Start)
                .unwrap();
        });
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

    fn confirm_solution_time(
        &self,
        ctx: &ContextWrapper<T>,
        history: Vec<HistoryAlias<T>>,
        elapsed: u32,
        mode: SearchMode,
    ) -> Arc<Solution<T>> {
        let mut confirm = self.startctx.clone();
        confirm = confirm.try_replay_all(self.world, history.iter()).unwrap();
        if confirm.elapsed() == elapsed {
            return confirm.into_solution();
        }
        if confirm.elapsed() < elapsed {
            return confirm.into_solution();
        }
        log::debug!(
            "Solution({:?}) elapsed time from db {}ms is better than history! {}ms. Checking for discrepancies...",
            mode,
            elapsed,
            confirm.elapsed()
        );
        if confirm.get() != ctx.get() {
            log::warn!(
                "Internal states differ: -db +recreated\n{}",
                confirm.get().diff(ctx.get())
            );
        }
        let (new_history, new_elapsed) = self.queue.db().get_history(confirm.get()).unwrap();
        if new_elapsed != elapsed {
            log::warn!(
                "Db read of recreated got new time: orig={}ms recreated={}ms",
                elapsed,
                new_elapsed
            );
        }
        if new_history != history {
            let old_hist = history_str::<T, _>(history.iter().copied());
            let new_hist = history_str::<T, _>(new_history.iter().copied());
            let text_diff = TextDiff::from_lines(&old_hist, &new_hist);
            log::warn!(
                "Route diff:\n{}",
                text_diff
                    .unified_diff()
                    .context_radius(3)
                    .header("db read orig", "db read recreated")
            );
        }
        let mut replay = self.startctx.clone();
        for (i, step) in new_history.into_iter().enumerate() {
            replay.assert_and_replay(self.world, step);
            assert!(
                !self.world.won(replay.get()),
                "Replay finished without finding a discrepancy"
            );
            let (db_hist, db_elapsed) = self.queue.db().get_history(replay.get()).unwrap();
            if replay.elapsed() != db_elapsed {
                log::debug!(
                    "Replay differs from db at step {}. {}\n{}ms replayed vs {}ms in db",
                    i,
                    step,
                    replay.elapsed(),
                    db_elapsed
                );
                let mut partial = self.startctx.clone();
                partial = partial.try_replay_all(self.world, db_hist.iter()).unwrap();
                if partial.elapsed() != db_elapsed {
                    log::debug!(
                        "Replaying partial still does not match: {}ms vs {}ms",
                        partial.elapsed(),
                        db_elapsed
                    );
                    if partial.recent_history() == replay.recent_history() {
                        log::debug!("History was the same despite discrepancy.");
                    };
                }

                let (history, new_elapsed) = self.queue.db().get_history(ctx.get()).unwrap();

                let retry = self.startctx.clone();
                let solution = retry
                    .try_replay_all(self.world, history.iter())
                    .unwrap()
                    .into_solution();
                log::debug!(
                    "Replacing solution({:?}) with history from second read: {}ms (previously: {}ms, {}ms)",
                    mode,
                    new_elapsed,
                    elapsed,
                    confirm.elapsed(),
                );
                return solution;
            }
        }
        panic!("Replay finished without winning or finding discrepancy");
    }

    fn handle_one_solution_and_minimize(
        &self,
        ctx: &mut ContextWrapper<T>,
        prev: &Option<T>,
        mode: SearchMode,
    ) -> Option<ContextWrapper<T>> {
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

        let (history, elapsed) = self.queue.db().get_history(ctx.get()).unwrap();

        let solution = self.confirm_solution_time(ctx, history, elapsed, mode);
        let elapsed = solution.elapsed;

        let mut sols = self.solutions.lock().unwrap();
        if iters > 10_000_000 || sols.unique() > 1_000 {
            self.queue.set_max_time(elapsed + elapsed / 8_192);
        } else if iters > 5_000_000 || sols.unique() > 100 {
            self.queue.set_max_time(elapsed + elapsed / 1_000);
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
            if mode != SearchMode::Minimized {
                return pinpoint_minimize::<_, _, _, TM>(self.world, self.startctx.get(), solution);
            }
        } else if res != SolutionResult::Duplicate && mode != SearchMode::Minimized {
            // Minimize against itself to see if it improves enough for inclusion
            return pinpoint_minimize::<_, _, _, TM>(self.world, self.startctx.get(), solution);
        }
        None
    }

    fn handle_solution(&self, ctx: &mut ContextWrapper<T>, prev: &Option<T>, mode: SearchMode) {
        if let Some(ctx) = self.handle_one_solution_and_minimize(ctx, prev, mode) {
            let solution = ctx.into_solution();
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
                    if self.world.won(ctx.get()) {
                        if ctx.elapsed() < max_time {
                            solutions.push(ctx);
                        }
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
                    self.handle_solution(&mut ctx, prev, mode);
                }
            },
        )
        .0
    }

    fn single_step(&self, ctx: ContextWrapper<T>) -> Vec<ContextWrapper<T>> {
        single_step(self.world, ctx, UNREASONABLE_TIME)
    }

    fn recreate_step(&self, ctx: ContextWrapper<T>) -> Vec<ContextWrapper<T>> {
        single_step_with_local(self.world, ctx, UNREASONABLE_TIME)
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

    /// Recreates the route from the given starting point, evaluating each state in the route onward if needed,
    /// and updating all the children of those states in the db.
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
            let next_steps = self.queue.db().get_next_steps(ctx.get()).unwrap();
            let prev = Some(ctx.get().clone());
            let elapsed = ctx.elapsed();
            let time_since_visit = ctx.time_since_visit();
            if !next_steps.is_empty() {
                let next = next_steps
                    .into_iter()
                    .filter_map(|vh| {
                        if vh.len() > 1 || vh[0] != *hist {
                            let cc = ctx.clone();
                            Some(cc.try_replay_all(self.world, vh.iter()).unwrap())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                ctx.assert_and_replay(self.world, *hist);
                self.queue.extend_keep_one(next, &ctx, &prev)?;
                ctx.remove_history();
            } else {
                let mut next = self.single_step(ctx.clone());
                if let Some((ci, _)) = next
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.recent_history().last() == Some(hist))
                    .min_by_key(|(_, c)| c.elapsed())
                {
                    // Assumption: no subsequent state leads to victory (aside from the last state?)
                    ctx = next.swap_remove(ci);
                    self.queue.extend_keep_one(next, &ctx, &prev)?;
                    ctx.remove_history();
                } else if matches!(hist, History::L(..)) {
                    ctx.assert_and_replay(self.world, *hist);
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
            let incr_organic = |progress, iters| {
                let org = self.organic_level.load(Ordering::Acquire);
                if org == progress || Some(org) <= self.queue.min_progress() {
                    self.organic_level
                        .fetch_max(progress + 1, Ordering::Release);
                    self.last_organic_improvement
                        .fetch_max(iters, Ordering::Release);
                }
            };

            while !self.finished.load(Ordering::Acquire)
                && workers_done.load(Ordering::Acquire) < num_workers
            {
                let iters = self.iters.load(Ordering::Acquire);
                let current_mode = if iters < 200_000 {
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
                    SearchMode::GreedyMax => self.queue.pop_max_progress(2),
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
                                let found = AtomicBool::default();

                                rayon::join(
                                    || {
                                        let ct = in_comm.len();
                                        in_comm.into_par_iter().for_each(|loc| {
                                            match self.process_one_greedy(
                                                &ctx,
                                                loc,
                                                max_time,
                                                current_mode,
                                                iters,
                                            ) {
                                                Ok(true) => {
                                                    found.store(true, Ordering::Release);
                                                    incr_organic(progress, iters)
                                                }
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
                                                    iters,
                                                ) {
                                                    Ok(true) => {
                                                        found.store(true, Ordering::Release);
                                                        Some(incr_organic(progress, iters))
                                                    }
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
                                if !found.load(Ordering::Acquire) {
                                    self.greedy_misses.fetch_add(1, Ordering::Release);
                                }
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
                                            incr_organic(visits, iters);
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
        // Profiler handler
        let rt = tokio::runtime::Runtime::new()?;
        #[cfg(all(feature = "jemalloc", not(target_env = "msvc")))]
        rt.spawn(async {
            let app = axum::Router::new()
                .route(
                    "/debug/pprof/heap",
                    axum::routing::get(jemalloc::handle_get_heap),
                );

            // run our app with hyper, listening globally on port 3000
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });
        rayon::scope(|scope| {
            // Background queue restore
            scope.spawn(|_| {
                self.queue.db().restore();
            });

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
                        if sol.elapsed > sols.cutoff() {
                            // We should have dropped these from the collector already.
                            continue;
                        }
                        drop(sols);

                        log::debug!(
                            "Solution mutator starting minimize for solution {}ms",
                            sol.elapsed
                        );
                        if let Some(min_ctx) = trie_minimize(
                            self.world,
                            self.startctx.get(),
                            sol.clone(),
                            &self.solve_trie,
                        ) {
                            self.recreate_store(
                                &self.startctx,
                                min_ctx.recent_history(),
                                SearchMode::MutateMinimize,
                            )
                            .unwrap();
                        }
                        if self.finished.load(Ordering::Acquire) {
                            return;
                        }
                        log::debug!(
                            "Solution mutator starting greedy-collection-steps for solution {}ms",
                            sol.elapsed
                        );
                        if let Some(min_ctx) = mutate_greedy_collections(
                            self.world,
                            self.startctx.get(),
                            self.queue.max_time(),
                            self.options.mutate_max_depth,
                            self.options.mutate_max_states,
                            sol.clone(),
                            self.queue.db().scorer().get_algo(),
                            &self.direct_paths,
                        ) {
                            self.recreate_store(
                                &self.startctx,
                                min_ctx.recent_history(),
                                SearchMode::MutateGreedySteps,
                            )
                            .unwrap();
                        }
                        if self.finished.load(Ordering::Acquire) {
                            return;
                        }
                        log::debug!(
                            "Solution mutator starting canon replacement for solution {}ms",
                            sol.elapsed
                        );
                        if let Some(min_ctx) = mutate_canon_locations(
                            self.world,
                            self.startctx.get(),
                            self.queue.max_time(),
                            self.options.mutate_max_depth,
                            self.options.mutate_max_states,
                            sol.clone(),
                            self.queue.db().scorer().get_algo(),
                            &self.direct_paths,
                        |min_ctx| {
                            self.recreate_store(
                                &self.startctx,
                                min_ctx.recent_history(),
                                SearchMode::MutateCanonLocations
                            )
                            .unwrap();
                        }) {
                            log::debug!("Solution mutator best found was {}ms", min_ctx.elapsed());
                        }
                        if self.finished.load(Ordering::Acquire) {
                            return;
                        }
                        log::debug!(
                            "Solution mutator starting revisits for solution {}ms",
                            sol.elapsed
                        );
                        let revisits =
                            mutate_spot_revisits(self.world, self.startctx.get(), sol.clone());
                        log::debug!(
                            "Solution mutator got {} revisits for solution {}ms",
                            revisits.len(),
                            sol.elapsed,
                        );
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
                        if self.finished.load(Ordering::Acquire) {
                            return;
                        }
                        log::debug!(
                            "Solution mutator starting reordering for solution {}ms",
                            sol.elapsed
                        );
                        let elapsed = sol.elapsed;
                        if let Some(reordered) = mutate_collection_steps(
                            self.world,
                            self.startctx.get(),
                            self.queue.max_time(),
                            self.options.mutate_max_depth,
                            self.options.mutate_max_states,
                            sol,
                            self.queue.db().scorer().get_algo(),
                            &self.direct_paths,
                        ) {
                            log::debug!(
                                "Solution mutator got a reordered solution for solution {}ms",
                                elapsed
                            );
                            self.recreate_store(
                                &self.startctx,
                                reordered.recent_history(),
                                SearchMode::MutateCollections,
                            )
                            .unwrap();
                        } else {
                            log::debug!(
                                "Solution mutator did not get a reordered solution for solution {}ms",
                                elapsed
                            );
                        }
                        self.mutated.fetch_add(1, Ordering::Release);
                        if self.finished.load(Ordering::Acquire) {
                            return;
                        }
                        sols = self.solutions.lock().unwrap();
                    }
                    (sols, _) = self
                        .solution_cvar
                        .wait_timeout(sols, max_wait_time)
                        .unwrap();
                    if self.finished.load(Ordering::Acquire) {
                        return;
                    }
                }
            });

            if self.queue.is_empty() && self.queue.db().recovery() {
                log::debug!("Waiting a bit for queue recovery...");
                std::thread::sleep(Duration::from_secs(10));
                for _ in 0..9 {
                    if !self.queue.db().is_empty() || !self.queue.db().recovery() {
                        break;
                    }
                    std::thread::sleep(Duration::from_secs(10));
                }
                assert!(
                    !self.queue.db().is_empty(),
                    "No queue recovery in 100 seconds, giving up"
                );
            }

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
        rt.shutdown_background();
        self.solutions.lock().unwrap().export()
    }

    fn process_one_greedy(
        &self,
        ctx: &ContextWrapper<T>,
        loc: &W::Location,
        max_time: u32,
        current_mode: SearchMode,
        iters: usize,
    ) -> anyhow::Result<bool> {
        let res = if current_mode == SearchMode::GreedyMax {
            // scale max depth and max states up if we haven't seen a lot for a while
            let last = self.last_organic_improvement.load(Ordering::Acquire);
            let time_since = std::cmp::min(iters.saturating_sub(last), 1_024_576);
            access_location_after_actions(
                self.world,
                ctx.clone(),
                loc.id(),
                max_time,
                std::cmp::max(1, self.options.greedy_max_depth * time_since / 1_024_576),
                std::cmp::max(
                    self.options.greedy_max_states / 16,
                    self.options.greedy_max_states * time_since / 1_024_576,
                ),
                self.queue.db().scorer().get_algo(),
                &self.direct_paths,
            )
        } else {
            access_location_after_actions(
                self.world,
                ctx.clone(),
                loc.id(),
                max_time,
                self.options.local_max_depth,
                self.options.local_max_states,
                self.queue.db().scorer().get_algo(),
                &self.direct_paths,
            )
        };
        let found = res.is_success();
        match res {
            AccessResult::SuccessfulAccess(mut c)
            | AccessResult::CachedPathMinSuccess(mut c)
            | AccessResult::CachedPathMinWithoutAccess(mut c)
            | AccessResult::CachedPathSuccess(mut c)
            | AccessResult::CachedPathWithoutAccess(mut c)
            | AccessResult::ReachedSpot(mut c) => {
                if !found {
                    self.greedy_spots_only.fetch_add(1, Ordering::Release);
                }
                let hist = c.remove_history().0;
                if !hist.is_empty() {
                    self.recreate_store(&ctx, &hist, current_mode)
                        .map(|_| found)
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

        if let Some(win) = trie_search(self.world, &ctx, UNREASONABLE_TIME, &self.solve_trie) {
            // Handles recording the solution, updating all steps, and single stepping as well.
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
        if best > 0 {
            if unique > 1_000 || iters > 10_000_000 {
                self.queue.set_max_time(best + best / 8_192);
            } else if iters > 5_000_000 || unique > 100 {
                self.queue.set_max_time(best + best / 1_000);
            } else if iters > 2_000_000 && unique > 4 {
                self.queue.set_max_time(best + best / 100);
            }
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
        let (num_routes, trie_size) = self.direct_paths.totals();
        println!(
            "--- Round {} (solutions={}, unique={}, mut={}, limit={}ms, best={}ms) ---\n\
            Stats: heap={}; pending={}; db={}; total={}; seen={}; proc={}; dead-end={}\n\
            trie size={}, depth={}, values={}; estimates={}; cached={}; evictions={}; retrievals={}\n\
            direct paths: hits={}, min hits={}, improves={}, fails={}, expires={}, deadends={}; routes={}, trie size={}\n\
            Greedy stats: org level={}, steps done={}, misses={}, spots={}, proc_in={}, proc_out={}\n\
            skips: push:{} time, {} dups; pop: {} time, {} dups; readds={}; bgdel={}\n\
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
            self.direct_paths.hits.load(Ordering::Acquire),
            self.direct_paths.min_hits.load(Ordering::Acquire),
            self.direct_paths.improves.load(Ordering::Acquire),
            self.direct_paths.fails.load(Ordering::Acquire),
            self.direct_paths.expires.load(Ordering::Acquire),
            self.direct_paths.deadends.load(Ordering::Acquire),
            num_routes,
            trie_size,
            self.organic_level.load(Ordering::Acquire),
            self.greedies.load(Ordering::Acquire),
            self.greedy_misses.load(Ordering::Acquire),
            self.greedy_spots_only.load(Ordering::Acquire),
            self.greedy_in_comm.load(Ordering::Acquire),
            self.greedy_out_comm.load(Ordering::Acquire),
            iskips,
            dskips,
            pskips,
            dpskips,
            self.queue.db().readds(),
            self.queue.background_deletes(),
            heap_bests.iter().position(|x| *x != None).unwrap_or(0),
            heap_bests.len().saturating_sub(1),
            heap_bests
                .into_iter()
                .map(|n| match n {
                    Some(score) => self.queue.db().metric().score_primary(score).to_string(),
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
