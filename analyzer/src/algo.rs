use crate::access::*;
use crate::context::*;
use crate::greedy::*;
use crate::heap::LimitedHeap;
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
    if locs.is_empty() && exit == None {
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

    if let Some((l, e)) = exit {
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
            c2.penalize(penalty * 2);
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

pub fn classic_step<W, T, L, E>(world: &W, ctx: ContextWrapper<T>, max_time: i32) -> Vec<ContextWrapper<T>>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<ExitId = L::ExitId, LocId = L::LocId, Context = T, Currency = L::Currency>,
{
    // The process will look more like this:
    // 1. explore -> vec of spot ctxs with penalties applied
    // 2. get largest dist
    // 3. (activate_actions) for each ctx, check for global actions and spot actions
    // 4. (visit_locations) for each ctx, get all available locations
    let spot_ctxs = explore(world, ctx, max_time);
    let mut with_locs = 0;
    let mut result = Vec::new();

    if let (Some(s), Some(f)) = (spot_ctxs.first(), spot_ctxs.last()) {
        let max_diff = f.elapsed() - s.elapsed();
        for ctx in spot_ctxs.iter() {
            let has_locs = spot_has_locations(world, ctx.get());
            // somewhat quadratic penalties
            let spot_penalty = with_locs * (with_locs - 1) * max_diff
                / <usize as TryInto<i32>>::try_into(spot_ctxs.len()).unwrap();
            if spot_has_actions(world, ctx) {
                result.extend(activate_actions(
                    world,
                    ctx,
                    if !has_locs {
                        6 * spot_penalty + 1000
                    } else {
                        3 * spot_penalty + 500
                    },
                ));
            }
            if has_locs {
                result.extend(visit_locations(world, ctx.clone(), spot_penalty));
                with_locs += 1;
            }
        }
    }
    result
}

fn choose_mode<T>(iters: i32, ctx: &ContextWrapper<T>, heap: &LimitedHeap<T>) -> SearchMode
where
    T: Ctx,
{
    if iters < 1_000_000 {
        SearchMode::Classic
    } else if iters % 1024 != 0 {
        SearchMode::Classic
    } else if ctx.elapsed() * 3 < heap.max_time() {
        SearchMode::Depth(4)
    } else if ctx.get().progress() > 60 {
        SearchMode::Greedy
    } else {
        SearchMode::PickDepth(3)
    }
}

fn handle_solution<T, W, L, E>(
    ctx: ContextWrapper<T>,
    heap: &mut LimitedHeap<T>,
    solutions: &mut SolutionCollector<T>,
    world: &W,
    startctx: &ContextWrapper<T>,
    iters: i32,
    mode: SearchMode,
) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<Context = T, ExitId = L::ExitId, LocId = L::LocId, Currency = L::Currency>,
{
    let old_time = heap.max_time();
    if iters > 10_000_000 && solutions.unique() > 4 {
        heap.set_max_time(ctx.elapsed());
    } else {
        heap.set_lenient_max_time(ctx.elapsed());
    }

    if solutions.len() == 0 || ctx.elapsed() < solutions.best() {
        println!(
            "{:?} mode found new shortest winning path after {} rounds: estimated {}ms (heap max was: {}ms)",
            mode,
            iters,
            ctx.elapsed(),
            old_time
        );
        println!("Max time to consider is now: {}ms", heap.max_time());

        // If we dropped to 80% of the previous max time
        if ctx.elapsed() * 10 < old_time * 8 {
            heap.clean();
        }
    }

    // If there were locations we skipped mid-route, skip them from the start,
    // in case that changes the routing.
    let newctx = ContextWrapper::new(remove_all_unvisited(world, startctx.get(), &ctx));
    heap.push(newctx);

    solutions.insert(ctx)
}

pub fn search<W, T, L, E>(world: &W, mut ctx: T) -> Result<(), std::io::Error>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<Context = T, ExitId = L::ExitId, LocId = L::LocId, Currency = L::Currency>,
{
    world.skip_unused_items(&mut ctx);
    let startctx = ContextWrapper::new(ctx);
    let mut heap = LimitedHeap::new();
    let mut solutions = SolutionCollector::<T>::new("data/solutions.txt", "data/previews.txt")?;
    let start = Instant::now();
    match greedy_search(world, &startctx, i32::MAX) {
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
            heap.set_lenient_max_time(wonctx.elapsed());
            heap.set_lenient_max_time(m.elapsed());
            heap.set_scale_factor(heap.max_time() / 18000);
            heap.push(ContextWrapper::new(remove_all_unvisited(
                world,
                startctx.get(),
                &m,
            )));

            solutions.insert(wonctx);
            solutions.insert(m);
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
    heap.push(startctx.clone());
    println!("Max time to consider is now: {}ms", heap.max_time());
    let mut iters = 0;
    let mut deadends = 0;
    let mut last_clean = heap.max_time();
    let mut last_solve = 0;
    let mut rescore_plus = true;
    let mut dist_for_rescoring = 1_000_000;
    let mut rescore_factor = 5;

    while let Some(ctx) = heap.pop() {
        // cut off when penalties are high enough
        // progressively raise the score threshold as the heap size increases
        let heapsize_adjustment: i32 = (heap.len() / 32).try_into().unwrap();
        let solutions_adjustment: i32 = solutions.len().try_into().unwrap();
        let score_cutoff: i32 = heapsize_adjustment - heap.max_time()
            + solutions_adjustment
            + if iters > 10_000_000 {
                (iters - 10_000_000) / 1_024
            } else {
                0
            };
        if ctx.score(heap.scale_factor()) < score_cutoff {
            println!(
                "Remaining items have low score: score={} vs max_time={}ms",
                ctx.score(heap.scale_factor()),
                heap.max_time()
            );
            break;
        }
        if heap.len() > 15_000_000 || heap.len() + heap.seen() > 50_000_000 {
            println!(
                "Too many items in heap! score={} vs adjusted={}",
                ctx.score(heap.scale_factor()),
                heapsize_adjustment - heap.max_time()
            );
            break;
        }
        if ctx.get().count_visits() + ctx.get().count_skips() >= W::NUM_LOCATIONS {
            deadends += 1;
            continue;
        }

        iters += 1;
        if iters % 10000 == 0 {
            if iters > 10_000_000 && solutions.unique() > 4 {
                heap.set_max_time(solutions.best());
            }
            if iters > 2_000_000 && iters - last_solve > dist_for_rescoring {
                println!("Rescoring.");
                if rescore_plus && heap.len() > 2_000_000 {
                    let new_factor = rescore_factor * heap.scale_factor() / 4;
                    if new_factor > 1_000_000 {
                        println!("Scale factor too high!");
                        break;
                    }
                    heap.set_scale_factor(new_factor);
                    if iters - last_solve > dist_for_rescoring {
                        dist_for_rescoring += 1_500_000;
                    }
                    rescore_plus = false;
                } else {
                    heap.set_scale_factor(4 * heap.scale_factor() / rescore_factor);
                    rescore_plus = true;
                }
                rescore_factor += 1;
                last_clean = heap.max_time();
                last_solve = iters;
            }
            if iters % 1_000_000 == 0 && heap.len() > 4_000_000 && heap.max_time() < last_clean {
                heap.clean();
                last_clean = heap.max_time();
            }
            let (iskips, pskips, dskips, dpskips) = heap.stats();
            println!(
                "--- Round {} (solutions: {}, unique: {}, dead-ends: {}, score cutoff: {}) ---\n\
                Heap stats: count={}; seen={}; current limit: {}ms, score scale factor: {}\npush_skips={} time + {} dups; pop_skips={} time + {} dups\n\
                {}",
                iters,
                solutions.len(),
                solutions.unique(),
                deadends,
                heapsize_adjustment - heap.max_time(),
                heap.len(),
                heap.seen(),
                heap.max_time(),
                heap.scale_factor(),
                iskips,
                dskips,
                pskips,
                dpskips,
                ctx.info(heap.scale_factor())
            );
        }

        let mode = choose_mode(iters, &ctx, &heap);
        match mode {
            SearchMode::Depth(d) if d > 1 => {
                depth_step(
                    world,
                    ctx,
                    &mut heap,
                    &mut solutions,
                    &startctx,
                    iters,
                    mode,
                    &mut dist_for_rescoring,
                    &mut last_solve,
                    d,
                );
            }
            SearchMode::Greedy => {
                if let Ok(win) = greedy_search(world, &ctx, heap.max_time()) {
                    if win.elapsed() <= heap.max_time() {
                        if handle_solution(
                            win,
                            &mut heap,
                            &mut solutions,
                            world,
                            &startctx,
                            iters,
                            mode,
                        ) {
                            dist_for_rescoring = 1_000_000;
                            last_solve = iters;
                        }
                    }
                }
                for ctx in classic_step(world, ctx, heap.max_time()) {
                    if world.won(ctx.get()) {
                        if handle_solution(
                            ctx,
                            &mut heap,
                            &mut solutions,
                            world,
                            &startctx,
                            iters,
                            SearchMode::Classic,
                        ) {
                            dist_for_rescoring = 1_500_000;
                            last_solve = iters;
                        }
                    } else {
                        heap.push(ctx);
                    }
                }
            }
            SearchMode::PickDepth(d) if d > 1 => {
                let mut this_round = vec![ctx];
                while let Some(c) = heap.pop() {
                    this_round.push(c);
                    if this_round.len() > 9 {
                        break;
                    }
                }
                this_round.sort_unstable_by_key(|c| c.elapsed() - c.penalty());
                depth_step(
                    world,
                    this_round.pop().unwrap(),
                    &mut heap,
                    &mut solutions,
                    &startctx,
                    iters,
                    mode,
                    &mut dist_for_rescoring,
                    &mut last_solve,
                    d,
                );
                heap.extend(this_round);
            }
            _ => {
                for ctx in classic_step(world, ctx, heap.max_time()) {
                    if world.won(ctx.get()) {
                        if handle_solution(
                            ctx,
                            &mut heap,
                            &mut solutions,
                            world,
                            &startctx,
                            iters,
                            SearchMode::Classic,
                        ) {
                            dist_for_rescoring = 1_500_000;
                            last_solve = iters;
                        }
                    } else {
                        heap.push(ctx);
                    }
                }
            }
        }
    }
    let (iskips, pskips, dskips, dpskips) = heap.stats();
    println!(
        "Finished after {} rounds ({} dead-ends), skipped {}+{} pushes + {}+{} pops",
        iters, deadends, iskips, dskips, pskips, dpskips
    );
    solutions.export()
}

fn depth_step<W, T, L, E>(
    world: &W,
    ctx: ContextWrapper<T>,
    heap: &mut LimitedHeap<T>,
    solutions: &mut SolutionCollector<T>,
    startctx: &ContextWrapper<T>,
    iters: i32,
    mode: SearchMode,
    dist_for_rescoring: &mut i32,
    last_solve: &mut i32,
    mut d: u8,
) where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W> + Debug,
    L: Location<Context = T>,
    E: Exit<Context = T, ExitId = L::ExitId, LocId = L::LocId, Currency = L::Currency>,
{
    let mut next = Vec::new();
    for ctx in classic_step(world, ctx, heap.max_time()) {
        if world.won(ctx.get()) {
            if handle_solution(ctx, heap, solutions, world, &startctx, iters, mode) {
                *dist_for_rescoring = 1_500_000;
                *last_solve = iters;
            }
        } else {
            next.push(ctx);
        }
    }
    d -= 1;
    next.sort_unstable_by_key(|c| (c.get().progress(), -c.elapsed()));
    while let Some(ctx) = next.pop() {
        if !heap.see(&ctx) {
            continue;
        }

        heap.extend(next);
        next = Vec::new();
        for ctx in classic_step(world, ctx, heap.max_time()) {
            if world.won(ctx.get()) {
                if handle_solution(ctx, heap, solutions, world, &startctx, iters, mode) {
                    *dist_for_rescoring = 1_500_000;
                    *last_solve = iters;
                }
            } else {
                next.push(ctx);
            }
        }
        d -= 1;
        if d == 0 {
            heap.extend(next);
            break;
        }
    }
}
