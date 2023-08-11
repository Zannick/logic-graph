use crate::access::move_to;
use crate::context::*;
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

pub fn debug_route<W, T, L>(
    world: &W,
    startctx: &T,
    route: &str,
) -> Result<String, String>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    let hist = hist_from_string::<W, T, L>(route)?;
    let mut ctx = ContextWrapper::new(startctx.clone());
    let mut output: Vec<String> = Vec::new();

    for (i, h) in hist.into_iter().enumerate() {
        output.push(format!("== {}. {} ==", i, h));
        let next = step_from_route(ctx.clone(), i, h, world)?;
        output.push(next.get().diff(ctx.get()));
        ctx = next;
    }
    Ok(output.join("\n"))
}