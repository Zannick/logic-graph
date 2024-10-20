use crate::access::move_to;
use crate::context::*;
use crate::estimates::ContextScorer;
use crate::steiner::graph::*;
use crate::steiner::*;
use crate::world::{Exit, Location, World};
use lazy_static::lazy_static;
use std::str::FromStr;
use std::time::{Duration, Instant};
use yaml_rust::Yaml;

static IN_FULL: &str = "\nin full:\n";

pub(crate) fn find_route_in_solution_string(solution: &str) -> &str {
    if solution.starts_with("Solution") {
        if let Some(idx) = solution.find(IN_FULL) {
            return &solution[idx + IN_FULL.len()..];
        }
    }
    solution
}

pub(crate) fn hist_from_string<T>(route: &str) -> Result<Vec<HistoryAlias<T>>, String>
where
    T: Ctx,
{
    let mut hist: Vec<HistoryAlias<T>> = Vec::new();
    let route = find_route_in_solution_string(route);
    for line in route.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            hist.push(History::from_str(line)?);
        }
    }
    Ok(hist)
}

pub(crate) fn histlines_from_string<T>(route: &str) -> Result<Vec<(HistoryAlias<T>, &str)>, String>
where
    T: Ctx,
{
    let mut hist: Vec<(HistoryAlias<T>, &str)> = Vec::new();
    let route = find_route_in_solution_string(route);
    for line in route.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            hist.push((History::from_str(line)?, line));
        }
    }
    Ok(hist)
}

pub(crate) fn histlines_from_yaml_vec<T>(
    route: &Vec<Yaml>,
) -> Result<Vec<(HistoryAlias<T>, &str)>, String>
where
    T: Ctx,
{
    let mut hist: Vec<(HistoryAlias<T>, &str)> = Vec::new();
    for el in route {
        if let Some(line) = el.as_str() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                hist.push((History::from_str(line)?, line));
            }
        } else {
            return Err(format!("Expected string but got: {:?}", el));
        }
    }
    Ok(hist)
}

pub(crate) fn step_from_route<W, T>(
    mut ctx: ContextWrapper<T>,
    i: usize,
    h: HistoryAlias<T>,
    world: &W,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Result<ContextWrapper<T>, String>
where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
{
    let pos = Wrapper::get(&ctx).position();
    let start = Instant::now();
    match h {
        History::G(item, loc_id) | History::V(item, loc_id, ..) => {
            let spot_id = world.get_location_spot(loc_id);
            if pos != spot_id {
                ctx = move_to(world, ctx, spot_id, shortest_paths).map_err(|s| {
                    format!(
                        "Could not complete route step {}: couldn't reach {} from {}\n{}",
                        i, spot_id, pos, s
                    )
                })?;
            }
            if item == Default::default() {
                let item = world.get_location(loc_id).item();
                ctx.try_replay(world, History::G(item, loc_id))
                    .map_err(|s| format!("Could not complete route step {} {}:\n{}", i, h, s))?;
            } else {
                ctx.try_replay(world, h)
                    .map_err(|s| format!("Could not complete route step {} {}:\n{}", i, h, s))?;
            }
        }
        History::E(exit_id) => {
            let spot_id = world.get_exit_spot(exit_id);
            if pos != spot_id {
                ctx = move_to(world, ctx, spot_id, shortest_paths).map_err(|s| {
                    format!(
                        "Could not complete route step {}: couldn't reach {} from {}\n{}",
                        i, spot_id, pos, s
                    )
                })?;
            }
            ctx.try_replay(world, h)
                .map_err(|s| format!("Could not complete route step {} {}:\n{}", i, h, s))?;
        }
        History::L(spot_id) | History::C(spot_id, ..) => {
            ctx = move_to(world, ctx, spot_id, shortest_paths).map_err(|s| {
                format!(
                    "Could not complete route step {}: couldn't reach {} from {}\n{}",
                    i, spot_id, pos, s
                )
            })?;
        }
        History::W(..) => {
            ctx.try_replay(world, h)
                .map_err(|s| format!("Could not complete route step {} {}:\n{}", i, h, s))?;
        }
        History::A(action_id) => {
            let spot_id = world.get_action_spot(action_id);
            if spot_id != Default::default() && pos != spot_id {
                ctx = move_to(world, ctx, spot_id, shortest_paths).map_err(|s| {
                    format!(
                        "Could not complete route step {}: couldn't reach {} from {}\n{}",
                        i, spot_id, pos, s
                    )
                })?;
            }
            ctx.try_replay(world, h)
                .map_err(|s| format!("Could not complete route step {} {}:\n{}", i, h, s))?;
        }
    }
    let elapsed = start.elapsed();
    lazy_static! {
        static ref WARNING_DUR: Duration = Duration::from_millis(50);
    }
    if elapsed > *WARNING_DUR {
        log::warn!("Long route step {} from {} ({:?}): {}", i, pos, elapsed, h);
    }
    Ok(ctx)
}

pub fn route_from_string<W, T>(
    world: &W,
    startctx: &T,
    route: &str,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Result<ContextWrapper<T>, (ContextWrapper<T>, String)>
where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
{
    let mut ctx = ContextWrapper::new(startctx.clone());
    let hist = hist_from_string::<T>(route).map_err(|e| (ctx.clone(), e))?;
    for (i, h) in hist.into_iter().enumerate() {
        ctx = step_from_route(ctx.clone(), i, h, world, shortest_paths).map_err(|e| (ctx, e))?;
    }
    Ok(ctx)
}

pub fn route_from_yaml_string<W, T>(
    world: &W,
    startctx: &T,
    route: &Yaml,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Result<ContextWrapper<T>, String>
where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
{
    match route {
        Yaml::String(s) => route_from_string(world, startctx, s, shortest_paths).map_err(|e| e.1),
        _ => Err(format!("Value for route is not str: {:?}", route)),
    }
}

pub fn debug_route<W, T>(
    world: &W,
    startctx: &T,
    route: &str,
    scorer: &ContextScorer<
        W,
        <W::Exit as Exit>::SpotId,
        <W::Location as Location>::LocId,
        EdgeId<W>,
        ShortestPaths<NodeId<W>, EdgeId<W>>,
    >,
) -> Result<String, String>
where
    W: World,
    T: Ctx<World = W>,
    W::Location: Location<Context = T>,
{
    let histlines = histlines_from_string::<T>(route)?;
    let mut ctx = ContextWrapper::new(startctx.clone());
    let mut output: Vec<String> = Vec::new();
    let steps = histlines.len();
    let start = Instant::now();

    for (i, (h, line)) in histlines.into_iter().enumerate() {
        output.push(format!("== {}. {} ==", i + 1, line));
        let mut next = step_from_route(ctx.clone(), i, h, world, scorer.get_algo())?;
        output.push(history_str::<T, _>(next.remove_history().0.into_iter()));
        output.push(next.get().diff(ctx.get()));
        let est = scorer.estimate_remaining_time(next.get());
        let el: u64 = next.elapsed().into();
        output.push(format!(
            "progress={}, est={}, elapsed={}, score={}",
            scorer.required_visits(next.get()),
            est,
            el,
            el + est
        ));
        ctx = next;
    }
    output.push(format!("Elapsed: {}ms", ctx.elapsed()));
    if !world.won(ctx.get()) {
        output.push(format!(
            "Remaining items needed: {:?}",
            world.items_needed(ctx.get())
        ));
    }
    log::info!(
        "Completed route in {:?} (average {:?})",
        start.elapsed(),
        start.elapsed() / steps as u32
    );
    Ok(output.join("\n"))
}
