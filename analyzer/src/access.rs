//! Functions related to access graphs, and accessing locations.

use crate::context::*;
use crate::world::*;
use std::cmp::Reverse;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::collections::{BinaryHeap, HashMap};

/// Check whether there are available locations at this position.
pub fn spot_has_locations<W, T, L, E>(world: &W, ctx: &T) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    world
        .get_spot_locations(ctx.position())
        .iter()
        .any(|loc| ctx.todo(loc.id()) && loc.can_access(ctx))
}

/// Check whether there are available actions at this position, including global actions.
pub fn spot_has_actions<W, T, L, E>(world: &W, ctx: &ContextWrapper<T>) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    world
        .get_global_actions()
        .iter()
        .chain(world.get_spot_actions(ctx.get().position()))
        .any(|act| act.can_access(ctx.get()) && ctx.is_useful(act))
}

pub fn spot_has_locations_or_actions<W, T, L, E>(world: &W, ctx: &ContextWrapper<T>) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    spot_has_locations(world, ctx.get()) || spot_has_actions(world, ctx)
}

pub fn expand<W, T, E, Wp>(
    world: &W,
    ctx: &ContextWrapper<T>,
    dist_map: &HashMap<E::SpotId, ContextWrapper<T>>,
    spot_heap: &mut BinaryHeap<Reverse<ContextWrapper<T>>>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    Wp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    for spot in world.get_area_spots(ctx.get().position()) {
        if !dist_map.contains_key(spot) {
            let local = ctx.get().local_travel_time(*spot);
            if local < 0 {
                panic!(
                    "Could not travel within area: start={:?} dest={:?}",
                    ctx.get().position(),
                    spot
                );
            }
            let mut newctx = ctx.clone();
            newctx.get_mut().set_position(*spot);
            newctx.history.push(History::MoveLocal(*spot));
            newctx.elapse(local);
            spot_heap.push(Reverse(newctx));
        }
    }

    for exit in world.get_spot_exits(ctx.get().position()) {
        if !dist_map.contains_key(&exit.dest()) && exit.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.exit(exit);
            spot_heap.push(Reverse(newctx));
        }
    }

    for warp in world.get_warps() {
        if !dist_map.contains_key(&warp.dest(ctx.get())) && warp.can_access(ctx.get()) {
            let mut newctx = ctx.clone();
            newctx.warp(warp);
            spot_heap.push(Reverse(newctx));
        }
    }
}

/// Variant for expand which doesn't track history or time, ideal for beatability checks
// (Is it possible to combine with the other without making it slower?)
pub fn expand_simple<W, T, E, Wp>(
    world: &W,
    ctx: &T,
    spot_map: &HashMap<E::SpotId, T>,
    ctx_queue: &mut VecDeque<T>,
) where
    W: World<Exit = E, Warp = Wp>,
    T: Ctx<World = W>,
    E: Exit<Context = T>,
    Wp: Warp<Context = T, SpotId = E::SpotId>,
{
    let mut append = |spot| {
        // We're copying the whole context on every step, which is probably
        // super inefficient; we only really copy because position may be relevant
        // for connection checks. If we tracked the "context" state separately
        // from the item state, it might be less copying.
        let mut newctx = ctx.clone();
        newctx.set_position(spot);
        ctx_queue.push_back(newctx);
    };
    for spot in world.get_area_spots(ctx.position()) {
        if !spot_map.contains_key(spot) {
            append(*spot);
        }
    }

    for exit in world.get_spot_exits(ctx.position()) {
        if !spot_map.contains_key(&exit.dest()) && exit.can_access(ctx) {
            append(exit.dest());
        }
    }

    for warp in world.get_warps() {
        if !spot_map.contains_key(&warp.dest(ctx)) && warp.can_access(ctx) {
            append(warp.dest(ctx));
        }
    }
}

/// Explores outward from the current position.
pub fn accessible_spots<W, T, E>(
    world: &W,
    ctx: ContextWrapper<T>,
) -> HashMap<E::SpotId, ContextWrapper<T>>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T, Currency = <W::Location as Accessible>::Currency>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId, Currency = <W::Location as Accessible>::Currency>,
{
    // return: spotid -> ctxwrapper
    let mut dist_map = HashMap::new();
    let mut spot_heap = BinaryHeap::new();
    let pos = ctx.get().position();
    dist_map.insert(pos, ctx);

    expand(world, &dist_map[&pos], &dist_map, &mut spot_heap);
    while let Some(Reverse(spot_found)) = spot_heap.pop() {
        let pos = spot_found.get().position();
        if !dist_map.contains_key(&pos) {
            dist_map.insert(pos, spot_found);
            expand(world, &dist_map[&pos], &dist_map, &mut spot_heap);
        }
    }

    // TODO: sort by distance
    dist_map
}

/// Variant of `access` that does not write hist or time, ideal for beatability checks
pub fn access_simple<W, T, E>(world: &W, ctx: &T) -> HashMap<<E as Exit>::SpotId, T>
where
    W: World<Exit = E>,
    T: Ctx<World = W>,
    E: Exit<Context = T>,
    W::Warp: Warp<Context = T, SpotId = E::SpotId>,
{
    let pos = ctx.position();
    let mut spot_map = HashMap::new();
    let mut ctx_queue = VecDeque::new();
    spot_map.insert(pos, ctx.clone());

    expand_simple(world, &spot_map[&pos], &spot_map, &mut ctx_queue);
    while !ctx_queue.is_empty() {
        let spot_found = ctx_queue.pop_front().unwrap();
        let pos = spot_found.position();
        if !spot_map.contains_key(&pos) {
            spot_map.insert(pos, spot_found);
            expand_simple(world, &spot_map[&pos], &spot_map, &mut ctx_queue);
        }
    }

    spot_map
}

pub fn visitable_locations<'a, W, T, L, E>(
    world: &'a W,
    ctx: &T,
) -> (Vec<&'a L>, Option<(<L as Location>::LocId, E::ExitId)>)
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    let mut exit = None;
    let locs: Vec<&L> = world
        .get_spot_locations(ctx.position())
        .iter()
        .filter(|loc| {
            if !ctx.todo(loc.id()) || !loc.can_access(ctx) {
                return false;
            } else if exit == None {
                if let Some(e) = loc.exit_id() {
                    exit = Some((loc.id(), *e));
                    return false;
                }
            }
            true
        })
        .collect();
    (locs, exit)
}

pub fn visit_simple<W, T, L, E>(world: &W, ctx: &mut T) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    let mut ret = false;
    for (spot_id, spot_ctx) in access_simple(world, &ctx) {
        for act in world.get_spot_actions(spot_id) {
            if act.can_access(&spot_ctx) && act.has_effect(&spot_ctx) {
                act.perform(ctx);
            }
        }
        for loc in world.get_spot_locations(spot_id) {
            // Check can_access at the local spot_ctx, but pick up items
            // and perform other checks with the omnipresent context.
            if ctx.todo(loc.id()) && loc.can_access(&spot_ctx) && ctx.can_afford(loc.price()) {
                ctx.collect(loc.item());
                ctx.spend(loc.price());
                for canon_loc_id in world.get_canon_locations(loc.id()) {
                    if ctx.todo(canon_loc_id) {
                        ctx.skip(canon_loc_id);
                    }
                }
                ctx.visit(loc.id());
                ret = true;
            }
        }
    }
    ret
}

pub fn can_win<W, T, L, E>(world: &W, ctx: &T) -> bool
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    let mut ctx = ctx.clone();
    let mut acts_only = 0;
    while !world.won(&ctx) {
        if !visit_simple(world, &mut ctx) {
            acts_only += 1;
        } else {
            acts_only = 0;
        }
        if acts_only > 1 {
            return false;
        }
    }
    true
}

// TODO: move elsewhere?
pub fn minimize<W, T, L, E>(
    world: &W,
    startctx: &T,
    wonctx: &ContextWrapper<T>,
) -> ContextWrapper<T>
where
    W: World<Location = L, Exit = E>,
    T: Ctx<World = W>,
    L: Location<ExitId = E::ExitId, LocId = E::LocId> + Accessible<Context = T>,
    E: Exit + Accessible<Context = T>,
{
    let mut ctx = startctx.clone();
    let mut set = HashSet::new();
    // Gather locations from the playthrough
    for hist in wonctx.history.iter().rev() {
        match hist {
            History::Get(_, loc_id) => {
                set.insert(*loc_id);
            }
            History::MoveGet(_, exit_id) => {
                let ex = world.get_exit(*exit_id);
                if let Some(loc_id) = ex.loc_id() {
                    set.insert(*loc_id);
                }
            }
            _ => (),
        }
    }
    let set = set;

    // skip all locations not in the playthrough
    for loc in world.get_all_locations() {
        if set.contains(&loc.id()) {
            continue;
        }
        if ctx.todo(loc.id()) {
            ctx.skip(loc.id());
            if !can_win(world, &ctx) {
                ctx.reset(loc.id());
            }
        }
    }

    // skip remaining visited locations from last to first
    for hist in wonctx.history.iter().rev() {
        match hist {
            History::Get(_, loc_id) => {
                ctx.skip(*loc_id);
                if !can_win(world, &ctx) {
                    ctx.reset(*loc_id);
                }
            }
            History::MoveGet(_, exit_id) => {
                let ex = world.get_exit(*exit_id);
                if let Some(loc_id) = ex.loc_id() {
                    ctx.skip(*loc_id);
                    if !can_win(world, &ctx) {
                        ctx.reset(*loc_id);
                    }
                }
            }
            _ => (),
        }
    }

    ContextWrapper::new(ctx)
}
