use crate::access::move_to;
use crate::context::*;
use crate::estimates::ContextScorer;
use crate::world::{Exit, Location, World};
use std::str::FromStr;
use yaml_rust::Yaml;

pub(crate) fn hist_from_string<W, T, L>(route: &str) -> Result<Vec<HistoryAlias<T>>, String>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    let mut hist: Vec<HistoryAlias<T>> = Vec::new();
    for line in route.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            hist.push(History::from_str(line)?);
        }
    }
    Ok(hist)
}

pub(crate) fn histlines_from_string<W, T, L>(
    route: &str,
) -> Result<Vec<(HistoryAlias<T>, &str)>, String>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    let mut hist: Vec<(HistoryAlias<T>, &str)> = Vec::new();
    for line in route.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with('#') {
            hist.push((History::from_str(line)?, line));
        }
    }
    Ok(hist)
}

pub(crate) fn histlines_from_yaml_vec<W, T, L>(
    route: &Vec<Yaml>,
) -> Result<Vec<(HistoryAlias<T>, &str)>, String>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
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

pub(crate) fn step_from_route<W, T, L>(
    mut ctx: ContextWrapper<T>,
    i: usize,
    h: HistoryAlias<T>,
    world: &W,
) -> Result<ContextWrapper<T>, String>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    let pos = Wrapper::get(&ctx).position();
    match h {
        History::G(item, loc_id) => {
            let spot_id = world.get_location_spot(loc_id);
            if pos != spot_id {
                ctx = move_to(world, ctx, spot_id).unwrap_or_else(|| {
                    panic!(
                        "Could not complete route step {}: couldn't reach {} from {}",
                        i, spot_id, pos
                    )
                });
            }
            if item == Default::default() {
                let item = world.get_location(loc_id).item();
                ctx.replay(world, History::G(item, loc_id));
            } else {
                ctx.replay(world, h);
            }
        }
        History::H(item, exit_id) => {
            let spot_id = world.get_exit_spot(exit_id);
            if pos != spot_id {
                ctx = move_to(world, ctx, spot_id).unwrap_or_else(|| {
                    panic!(
                        "Could not complete route step {}: couldn't reach {} from {}",
                        i, spot_id, pos
                    )
                });
            }

            if item == Default::default() {
                let exit = world.get_exit(exit_id);
                if let Some(loc_id) = exit.loc_id() {
                    let item = world.get_location(*loc_id).item();
                    ctx.replay(world, History::H(item, exit_id));
                } else {
                    return Err(format!("Not a hybrid exit: {}", exit_id));
                }
            } else {
                ctx.replay(world, h);
            }
        }
        History::E(exit_id) => {
            let exit = world.get_exit(exit_id);
            ctx = move_to(world, ctx, exit.dest()).unwrap_or_else(|| {
                panic!(
                    "Could not complete route step {}: couldn't reach {}",
                    i,
                    exit.dest()
                )
            });
        }
        History::L(spot_id) | History::C(spot_id) => {
            ctx = move_to(world, ctx, spot_id).unwrap_or_else(|| {
                panic!(
                    "Could not complete route step {}: couldn't reach {} from {}",
                    i, spot_id, pos
                )
            });
        }
        History::W(..) => ctx.replay(world, h),
        History::A(action_id) => {
            let spot_id = world.get_action_spot(action_id);
            if spot_id != Default::default() && pos != spot_id {
                ctx = move_to(world, ctx, spot_id).unwrap_or_else(|| {
                    panic!(
                        "Could not complete route step {}: couldn't reach {} from {}",
                        i, spot_id, pos
                    )
                });
            }
            ctx.replay(world, h);
        }
    }
    Ok(ctx)
}

pub fn route_from_string<W, T, L>(
    world: &W,
    startctx: &T,
    route: &str,
) -> Result<ContextWrapper<T>, String>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    let hist = hist_from_string::<W, T, L>(route)?;
    let mut ctx = ContextWrapper::new(startctx.clone());
    for (i, h) in hist.into_iter().enumerate() {
        ctx = step_from_route(ctx, i, h, world)?;
    }
    Ok(ctx)
}

pub fn route_from_yaml_string<W, T, L>(
    world: &W,
    startctx: &T,
    route: &Yaml,
) -> Result<ContextWrapper<T>, String>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    match route {
        Yaml::String(s) => route_from_string(world, startctx, s),
        _ => Err(format!("Value for route is not str: {:?}", route)),
    }
}

pub fn debug_route<W, T, L>(world: &W, startctx: &T, route: &str) -> Result<String, String>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    let histlines = histlines_from_string::<W, T, L>(route)?;
    let mut ctx = ContextWrapper::new(startctx.clone());
    let scorer = ContextScorer::shortest_paths(world, startctx, 32_768);
    let mut output: Vec<String> = Vec::new();

    for (i, (h, line)) in histlines.into_iter().enumerate() {
        output.push(format!("== {}. {} ==", i + 1, line));
        let mut next = step_from_route(ctx.clone(), i, h, world)?;
        output.push(history_str::<T, _>(next.remove_history().0.into_iter()));
        output.push(next.get().diff(ctx.get()));
        let est = scorer.estimate_remaining_time(next.get());
        let el: u64 = next.elapsed().into();
        output.push(format!("progress={}, est={}, elapsed={}, score={}", scorer.required_visits(next.get()), est, el, el + est));
        ctx = next;
    }
    output.push(format!("Elapsed: {}ms", ctx.elapsed()));
    if !world.won(ctx.get()) {
        output.push(format!("Remaining items needed: {:?}", world.items_needed(ctx.get())));
    }
    Ok(output.join("\n"))
}
