use crate::access::*;
use crate::context::*;
use crate::greedy::*;
use crate::heap::RocksBackedQueue;
use crate::minimize::*;
use crate::solutions::SolutionCollector;
use crate::world::*;
use std::fmt::Debug;
use std::time::Instant;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum SearchMode {
    Classic,
    Depth(u8),
    Greedy,
    PickDepth(u8),
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

    vec.sort_unstable_by_key(|el| el.elapsed());
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
    penalty: i32,
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
            c2.penalize(penalty * 4);
            result.push(c2);
        }
    }
    for act in world.get_spot_actions(ctx.get().position()) {
        if act.can_access(ctx.get()) {
            let mut c2 = ctx.clone();
            c2.activate(act);
            c2.penalize(penalty);
            result.push(c2);
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
    solutions: SolutionCollector<T>,
    queue: RocksBackedQueue<T>,
    iters: i32,
    deadends: u32,
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

        let queue =
            RocksBackedQueue::new(".db", max_time + max_time / 10, 400_000, 100, 100_000).unwrap();
        queue.push(startctx.clone()).unwrap();
        queue.push(clean_ctx).unwrap();
        println!("Max time to consider is now: {}ms", queue.max_time());
        Ok(Search {
            world,
            startctx,
            solutions,
            queue,
            iters: 0,
            deadends: 0,
        })
    }

    fn handle_solution(&mut self, ctx: ContextWrapper<T>, mode: SearchMode) -> bool {
        let old_time = self.queue.max_time();
        if self.iters > 10_000_000 && self.solutions.unique() > 4 {
            self.queue.set_max_time(ctx.elapsed());
        } else {
            self.queue.set_lenient_max_time(ctx.elapsed());
        }

        if self.solutions.is_empty() || ctx.elapsed() < self.solutions.best() {
            println!(
                "{:?} mode found new shortest winning path after {} rounds: estimated {}ms (heap max was: {}ms)",
                mode,
                self.iters,
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

        self.solutions.insert(ctx)
    }

    fn classic_step(&mut self, ctx: ContextWrapper<T>) -> Vec<ContextWrapper<T>> {
        // The process will look more like this:
        // 1. explore -> vec of spot ctxs with penalties applied
        // 2. get largest dist
        // 3. (activate_actions) for each ctx, check for global actions and spot actions
        // 4. (visit_locations) for each ctx, get all available locations
        let spot_ctxs = explore(self.world, ctx, self.queue.max_time());
        let mut with_locs = 0;
        let mut result = Vec::new();

        if let (Some(s), Some(f)) = (spot_ctxs.first(), spot_ctxs.last()) {
            let max_diff = f.elapsed() - s.elapsed();
            for ctx in spot_ctxs.iter() {
                let has_locs = spot_has_locations(self.world, ctx.get());
                // somewhat quadratic penalties
                let spot_penalty = with_locs * (with_locs - 1) * max_diff
                    / <usize as TryInto<i32>>::try_into(spot_ctxs.len()).unwrap();
                if spot_has_actions(self.world, ctx) {
                    result.extend(activate_actions(
                        self.world,
                        ctx,
                        if !has_locs {
                            6 * spot_penalty + 1000
                        } else {
                            3 * spot_penalty + 500
                        },
                    ));
                }
                if has_locs {
                    result.extend(visit_locations(self.world, ctx.clone(), spot_penalty));
                    with_locs += 1;
                }
            }
        }
        result
    }

    fn depth_step(&mut self, ctx: ContextWrapper<T>, mode: SearchMode, mut d: u8) -> Option<i32> {
        let mut next = Vec::new();
        let mut min_score = Some(ctx.score(self.queue.scale_factor()));
        for ctx in self.classic_step(ctx) {
            if self.world.won(ctx.get()) {
                self.handle_solution(ctx, mode);
                min_score = None;
            } else {
                if let Some(c) = min_score {
                    min_score = Some(std::cmp::max(c, ctx.score(self.queue.scale_factor())));
                }
                next.push(ctx);
            }
        }
        d -= 1;
        next.sort_unstable_by_key(|c| (c.get().progress(), -c.elapsed()));
        while let Some(ctx) = next.pop() {
            self.queue.extend(next).unwrap();
            next = Vec::new();
            for ctx in self.classic_step(ctx) {
                if self.world.won(ctx.get()) {
                    self.handle_solution(ctx, mode);
                    min_score = None;
                } else {
                    if let Some(c) = min_score {
                        min_score = Some(std::cmp::max(c, ctx.score(self.queue.scale_factor())));
                    }
                    next.push(ctx);
                }
            }
            d -= 1;
            if d == 0 {
                self.queue.extend(next).unwrap();
                break;
            }
        }
        min_score
    }

    fn choose_mode(&self, ctx: &ContextWrapper<T>) -> SearchMode {
        if self.iters < 1_000_000 || self.iters % 2048 != 0 {
            SearchMode::Classic
        } else if ctx.elapsed() * 3 < self.queue.max_time() {
            SearchMode::Depth(4)
        } else if ctx.get().progress() > 60 {
            SearchMode::Greedy
        } else {
            SearchMode::PickDepth(3)
        }
    }

    pub fn search(mut self) -> Result<(), std::io::Error> {
        let pc = rocksdb::perf::PerfContext::default();

        while let Ok(Some(ctx)) = self.queue.pop() {
            // cut off when penalties are high enough
            // progressively raise the score threshold as the heap size increases
            let heapsize_adjustment: i32 = (self.queue.len() / 32).try_into().unwrap();
            let solutions_adjustment: i32 = self.solutions.len().try_into().unwrap();
            let score_cutoff: i32 = heapsize_adjustment - self.queue.max_time()
                + solutions_adjustment
                + if self.iters > 10_000_000 {
                    (self.iters - 10_000_000) / 1_024
                } else {
                    0
                };
            if ctx.score(self.queue.scale_factor()) < score_cutoff {
                println!(
                "Remaining items have low score: score={} (elapsed={}, penalty={}, factor={}) vs max_time={}ms\n{}",
                ctx.score(self.queue.scale_factor()),
                ctx.elapsed(),
                ctx.penalty(),
                self.queue.scale_factor(),
                self.queue.max_time(),
                ctx.info(self.queue.scale_factor())
            );
                self.queue.print_queue_histogram();
                break;
            }
            if ctx.get().count_visits() + ctx.get().count_skips() >= W::NUM_LOCATIONS {
                self.deadends += 1;
                continue;
            }

            self.iters += 1;
            if self.iters % 10000 == 0 {
                if self.iters > 10_000_000 && self.solutions.unique() > 4 {
                    self.queue.set_max_time(self.solutions.best());
                }
                if self.iters % 1_000_000 == 0 {
                    self.queue.print_queue_histogram();
                }
                let (iskips, pskips, dskips, dpskips) = self.queue.skip_stats();
                let max_time = self.queue.max_time();
                println!(
                "--- Round {} (solutions: {}, unique: {}, dead-ends: {}, score cutoff: {}, scale factor: {}) ---\n\
                Queue stats: heap={}; db={}; total={}; seen={}; current limit: {}ms; db best: {}\npush_skips={} time + {} dups; pop_skips={} time + {} dups\n\
                {}",
                self.iters,
                self.solutions.len(),
                self.solutions.unique(),
                self.deadends,
                heapsize_adjustment - max_time,
                self.queue.scale_factor(),
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
                ctx.info(self.queue.scale_factor())
            );
            }

            let mode = self.choose_mode(&ctx);
            match mode {
                SearchMode::Depth(d) if d > 1 => {
                    self.depth_step(ctx, mode, d);
                }
                SearchMode::Greedy => {
                    if let Ok(win) = greedy_search(self.world, &ctx, self.queue.max_time()) {
                        if win.elapsed() <= self.queue.max_time() {
                            self.handle_solution(win, mode);
                        }
                    }
                    let next: Vec<ContextWrapper<T>> = self
                        .classic_step(ctx)
                        .into_iter()
                        .filter_map(|ctx| {
                            if self.world.won(ctx.get()) {
                                self.handle_solution(ctx, SearchMode::Classic);
                                None
                            } else {
                                Some(ctx)
                            }
                        })
                        .collect();
                    self.queue.extend(next).unwrap();
                }
                SearchMode::PickDepth(d) if d > 1 => {
                    let mut this_round = vec![ctx];
                    while let Some(c) = self.queue.pop().unwrap() {
                        this_round.push(c);
                        if this_round.len() > 9 {
                            break;
                        }
                    }
                    this_round.sort_unstable_by_key(|c| c.elapsed() - c.penalty());
                    self.depth_step(this_round.pop().unwrap(), mode, d);
                    self.queue.extend(this_round).unwrap();
                }
                _ => {
                    let next: Vec<ContextWrapper<T>> = self
                        .classic_step(ctx)
                        .into_iter()
                        .filter_map(|ctx| {
                            if self.world.won(ctx.get()) {
                                self.handle_solution(ctx, SearchMode::Classic);
                                None
                            } else {
                                Some(ctx)
                            }
                        })
                        .collect();
                    self.queue.extend(next).unwrap();
                }
            }
        }
        let (iskips, pskips, dskips, dpskips) = self.queue.skip_stats();
        println!(
            "Finished after {} rounds ({} dead-ends), skipped {}+{} pushes + {}+{} pops",
            self.iters, self.deadends, iskips, dskips, pskips, dpskips
        );
        println!("{}", pc.report(true));
        self.solutions.export()
    }
}
