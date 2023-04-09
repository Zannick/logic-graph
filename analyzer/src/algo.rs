use crate::access::*;
use crate::context::*;
use crate::greedy::*;
use crate::heap::RocksBackedQueue;
use crate::minimize::*;
use crate::solutions::SolutionCollector;
use crate::world::*;
use rayon::prelude::*;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum SearchMode {
    Classic,
    Depth(u8),
    Greedy,
    PickDepth(u8),
    PickMinElapsed,
    PickMinScore,
    Dependent,
    Unknown,
}

fn mode_by_index(index: usize) -> SearchMode {
    match index % 16 {
        1 | 6 | 10 | 14 => SearchMode::Dependent,
        2 | 7 | 11 | 15 => SearchMode::Greedy,
        4 => SearchMode::Depth(3),
        8 => SearchMode::PickDepth(3),
        12 => SearchMode::PickDepth(8),
        5 => SearchMode::PickMinScore,
        _ => SearchMode::Classic,
    }
}

pub fn explore<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    max_time: i32,
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

pub fn visit_locations<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    penalty: i32,
) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<ExitId = L::ExitId, Context = T, Currency = L::Currency>,
{
    let mut ctx_list = vec![ctx];
    ctx_list[0].penalize(penalty);
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

pub fn activate_actions<W, T, L, E>(
    world: &W,
    ctx: &ContextWrapper<T>,
    local_penalty: i32,
    global_penalty: i32,
) -> Vec<ContextWrapper<T>>
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
                c2.penalize(global_penalty);
                result.push(c2);
            }
        }
    }
    for act in world.get_spot_actions(ctx.get().position()) {
        if act.can_access(ctx.get()) {
            let mut c2 = ctx.clone();
            c2.activate(act);
            if c2.get() != ctx.get() {
                c2.penalize(local_penalty);
                result.push(c2);
            }
        }
    }
    result
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
    iters: AtomicU64,
    extras: AtomicU64,
    deadends: AtomicU32,
}

impl<'a, W, T, L, E> Search<'a, W, T>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<Context = T, ExitId = L::ExitId, LocId = L::LocId, Currency = L::Currency>,
{
    pub fn new(world: &'a W, mut ctx: T) -> Result<Search<'a, W, T>, std::io::Error> {
        world.skip_unused_items(&mut ctx);
        let startctx = ContextWrapper::new(ctx);
        let mut solutions = SolutionCollector::<T>::new("data/solutions.txt", "data/previews.txt")?;
        let start = Instant::now();
        let (clean_ctx, max_time) = match greedy_search(world, &startctx, i32::MAX) {
            Ok(wonctx) => {
                println!("Finished greedy search in {:?}", start.elapsed());
                let start = Instant::now();
                let m = minimize_greedy(world, startctx.get(), &wonctx, wonctx.elapsed());
                println!("Minimized in {:?}", start.elapsed());
                println!(
                    "Found greedy solution of {}ms, minimized to {}ms",
                    wonctx.elapsed(),
                    m.elapsed()
                );
                let max_time = std::cmp::min(wonctx.elapsed(), m.elapsed());
                let clean_ctx =
                    ContextWrapper::new(remove_all_unvisited(world, startctx.get(), &m));

                solutions.insert(wonctx);
                solutions.insert(m);
                (clean_ctx, max_time)
            }
            Err(ctx) => {
                panic!(
                    "Found no greedy solution, maximal attempt reached dead-end after {}ms:\n{}\n{:#?}",
                    ctx.elapsed(),
                    ctx.history_summary(),
                    ctx.get()
                );
                // Push it anyway, maybe it'll find something!
                //heap.push(ctx);
            }
        };

        let queue = RocksBackedQueue::new(
            world,
            ".db",
            max_time + max_time / 10,
            1_048_576,
            1_024,
            131_072,
            1_024,
            8_096,
        )
        .unwrap();
        queue.push(startctx.clone()).unwrap();
        queue.push(clean_ctx).unwrap();
        println!("Max time to consider is now: {}ms", queue.max_time());
        Ok(Search {
            world,
            startctx,
            solutions: Mutex::new(solutions),
            queue,
            iters: 0.into(),
            extras: 0.into(),
            deadends: 0.into(),
        })
    }

    fn handle_solution(&self, ctx: ContextWrapper<T>, mode: SearchMode) -> bool {
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

        sols.insert(ctx)
    }

    fn handle_greedy_solution(
        &self,
        ctx: ContextWrapper<T>,
        fork: &ContextWrapper<T>,
        check_prior: bool,
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

        self.handle_solution(ctx, mode);

        if check_prior {
            let first_back = oldhistlen / 2;

            let mut prior = self.startctx.clone();
            for (i, step) in oldhist.iter().rev().enumerate() {
                prior.replay(self.world, *step);
                if i >= first_back && !matches!(step, History::Move(_) | History::MoveLocal(_)) {
                    if let Ok(win) = greedy_search(self.world, &prior, i32::MAX) {
                        self.handle_greedy_solution(win, &prior, false, mode);
                        newstates.push(prior.clone());
                    }
                }
            }
        }

        self.queue.extend(newstates).unwrap();
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
        // The process will look more like this:
        // 1. explore -> vec of spot ctxs with penalties applied
        // 2. get largest dist
        // 3. (activate_actions) for each ctx, check for global actions and spot actions
        // 4. (visit_locations) for each ctx, get all available locations
        let spot_ctxs = explore(self.world, ctx, self.queue.max_time());
        let mut result = Vec::new();

        if let (Some(s), Some(f)) = (spot_ctxs.first(), spot_ctxs.last()) {
            let max_diff = f.elapsed() - s.elapsed();
            let spots: Vec<_> = spot_ctxs
                .into_iter()
                .map(|ctx| (spot_has_locations(self.world, ctx.get()), ctx))
                .collect();
            let with_locs: i32 = (spots.iter().filter(|(l, _)| *l).count())
                .try_into()
                .unwrap();
            let spot_count: i32 = spots.len().try_into().unwrap();
            let mut locs_count = 0;
            for (has_locs, ctx) in spots {
                // somewhat quadratic penalties
                let loc_penalty = locs_count * (locs_count - 1) * max_diff / spot_count;
                // Max penalty at any spot with no locations
                let act_penalty = with_locs * (with_locs - 1) * max_diff / spot_count;
                if spot_has_actions(self.world, ctx.get()) {
                    result.extend(activate_actions(
                        self.world,
                        &ctx,
                        loc_penalty + 500,
                        if !has_locs {
                            act_penalty + 1000
                        } else {
                            2 * (loc_penalty + 500)
                        },
                    ));
                }
                if has_locs {
                    result.extend(visit_locations(self.world, ctx, loc_penalty));
                    locs_count += 1;
                }
            }
        }
        result
    }

    fn depth_step(
        &self,
        ctx: ContextWrapper<T>,
        mode: SearchMode,
        mut d: u8,
    ) -> Vec<ContextWrapper<T>> {
        let mut next = Vec::new();
        let mut ret = Vec::new();
        for ctx in self.classic_step(ctx) {
            if self.world.won(ctx.get()) {
                self.handle_solution(ctx, mode);
            } else {
                next.push(ctx);
            }
        }
        if next.is_empty() {
            return ret;
        }
        d -= 1;
        // No need to sort, when we only want to pop the max element by (progress, elapsed).
        crate::swap_max_to_end(&mut next, |c| (c.get().progress(), -c.elapsed()));
        while let Some(ctx) = next.pop() {
            ret.extend(next);
            next = Vec::new();
            self.extras.fetch_add(1, Ordering::Release);
            for ctx in self.classic_step(ctx) {
                if self.world.won(ctx.get()) {
                    self.handle_solution(ctx, mode);
                } else {
                    next.push(ctx);
                }
            }
            d -= 1;
            if d == 0 {
                ret.extend(next);
                break;
            }

            crate::swap_max_to_end(&mut next, |c| (c.get().progress(), -c.elapsed()));
        }
        ret
    }

    fn choose_mode(&self, iters: u64, ctx: &ContextWrapper<T>) -> SearchMode {
        if iters % 2048 != 0 {
            SearchMode::Classic
        } else if iters % 4096 == 0 {
            SearchMode::PickMinElapsed
        } else if ctx.elapsed() * 3 < self.queue.max_time() {
            SearchMode::Depth(4)
        } else {
            SearchMode::PickDepth(3)
        }
    }

    pub fn search(self) -> Result<(), std::io::Error> {
        let start = Mutex::new(Instant::now());
        println!(
            "Starting search with {} threads",
            rayon::current_num_threads()
        );

        struct Iter<'a, W, T: Ctx> {
            q: &'a RocksBackedQueue<'a, W, T>,
        }
        impl<'a, W, T> Iterator for Iter<'a, W, T>
        where
            W: World,
            T: Ctx<World = W>,
        {
            type Item = ContextWrapper<T>;

            fn next(&mut self) -> Option<Self::Item> {
                self.q.pop().unwrap()
            }
        }

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

        if iters % 10000 == 0 {
            self.print_status_update(&start, iters, 10000, &ctx);
        }

        let current_mode = if iters < 500_000 {
            SearchMode::Classic
        } else if mode == SearchMode::Dependent {
            self.choose_mode(iters, &ctx)
        } else {
            mode
        };
        match current_mode {
            SearchMode::Depth(d) if d > 1 => {
                self.depth_step(ctx, mode, d);
                Ok(Vec::new())
            }
            SearchMode::Greedy => {
                // Run a classic step on the given state and handle any solutions
                let next = self.extract_solutions(self.classic_step(ctx), SearchMode::Classic);
                // Pick a state greedily: max progress, min elapsed, and do a greedy search.
                if let Some(ctx) = next
                    .iter()
                    .max_by_key(|ctx| (ctx.get().progress(), -ctx.elapsed()))
                {
                    if let Ok(win) = greedy_search(self.world, &ctx, self.queue.max_time()) {
                        if win.elapsed() <= self.queue.max_time() {
                            self.handle_greedy_solution(win, &ctx, true, mode);
                        }
                    }
                }

                // All the classic states are still pushed to the queue, even the one we used
                self.extras.fetch_add(1, Ordering::Release);
                Ok(next)
            }
            SearchMode::PickDepth(d) if d > 1 => {
                let mut this_round = vec![ctx];
                while let Some(c) = self.queue.pop().unwrap() {
                    this_round.push(c);
                    if this_round.len() > 9 {
                        break;
                    }
                }
                crate::swap_min_to_end(&mut this_round, |c| (c.elapsed(), c.penalty()));
                self.depth_step(this_round.pop().unwrap(), mode, d);
                Ok(this_round)
            }
            SearchMode::PickMinScore => {
                let mut next = self.extract_solutions(self.classic_step(ctx), SearchMode::Classic);
                if let Some(ctx2) = self.queue.pop_min_score().unwrap() {
                    next.extend(
                        self.extract_solutions(self.classic_step(ctx2), SearchMode::PickMinScore),
                    );
                }
                self.extras.fetch_add(1, Ordering::Release);
                Ok(next)
            }
            SearchMode::PickMinElapsed => {
                if let Some(minctx) = self.queue.pop_min_elapsed().unwrap() {
                    if ctx.elapsed() < minctx.elapsed() {
                        let mut next = self.extract_solutions(self.classic_step(ctx), mode);
                        next.push(minctx);
                        Ok(next)
                    } else {
                        let mut next = self.extract_solutions(self.classic_step(minctx), mode);
                        next.push(ctx);
                        Ok(next)
                    }
                } else {
                    let next = self.extract_solutions(self.classic_step(ctx), mode);
                    Ok(next)
                }
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
        iters: u64,
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
            "--- Round {} (ex: {}, solutions: {}, unique: {}, dead-ends: {}) ---\n\
                    Queue stats: heap={}; db={}; total={}; seen={}; limit: {}ms; db best: {}\n\
                    push_skips={} time + {} dups; pop_skips={} time + {} dups; evictions: {}; retrievals: {}\n\
                    {}",
            iters,
            self.extras.load(Ordering::Acquire),
            sols.len(),
            sols.unique(),
            self.deadends.load(Ordering::Acquire),
            self.queue.heap_len(),
            self.queue.db_len(),
            self.queue.len(),
            self.queue.seen(),
            max_time,
            self.queue.db_best(),
            iskips,
            dskips,
            pskips,
            dpskips,
            self.queue.evictions(),
            self.queue.retrievals(),
            ctx.info()
        );
    }
}
