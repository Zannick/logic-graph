//! AUTOGENERATED FOR Axiom Verge 2 - MODIFICATIONS WILL BE LOST

#![allow(unused)]
#![allow(non_snake_case)]

use analyzer::context::Ctx;
use analyzer::world::*;
use analyzer::*;
use libaxiom_verge2::context::{enums, flags, Context, Status};
use libaxiom_verge2::graph::{self, *};
use libaxiom_verge2::items::Item;

fn shared_setup() -> (graph::World, Context) {
    let mut world = graph::World::new();
    let mut ctx = Context::default();
    ctx.cbits2.insert(flags::ContextBits2::ICE_AXE);
    ctx.cbits1.insert(flags::ContextBits1::AMASHILAMA);
    ctx.cbits1.insert(flags::ContextBits1::BOOMERANG);
    ctx.cbits2.insert(flags::ContextBits2::LEDGE_GRAB);
    ctx.infect = 3;
    ctx.cbits2.insert(flags::ContextBits2::REMOTE_DRONE);
    ctx.cbits2.insert(flags::ContextBits2::SHOCKWAVE);
    ctx.cbits2.insert(flags::ContextBits2::UNDERWATER_MOVEMENT);
    ctx.energy = 300;
    ctx.save = SpotId::Giguna__Giguna_Northeast__Save_Point;

    (world, ctx)
}

#[test]
fn start_Giguna__Giguna_Northeast__Save_Point_can_reach_Giguna__Carnelian__Upper_Susar() {
    let (mut world, mut ctx) = shared_setup();

    expect_any_route!(
        &world,
        ctx,
        SpotId::Giguna__Giguna_Northeast__Save_Point,
        SpotId::Giguna__Carnelian__Upper_Susar
    );
}
#[test]
fn start_Giguna__Carnelian__Upper_Susar_context_giguna__carnelian__ctx__upper_susar_True_can_reach_Giguna__West_Caverns__East_Susar(
) {
    let (mut world, mut ctx) = shared_setup();
    ctx.cbits1
        .insert(flags::ContextBits1::GIGUNA__CARNELIAN__CTX__UPPER_SUSAR);

    expect_any_route!(
        &world,
        ctx,
        SpotId::Giguna__Carnelian__Upper_Susar,
        SpotId::Giguna__West_Caverns__East_Susar
    );
}
#[test]
fn start_Giguna__Carnelian__Upper_Susar_context_giguna__carnelian__ctx__upper_susar_True_can_obtain_Wall_Climb(
) {
    let (mut world, mut ctx) = shared_setup();
    ctx.cbits1
        .insert(flags::ContextBits1::GIGUNA__CARNELIAN__CTX__UPPER_SUSAR);

    expect_obtainable!(
        &world,
        ctx,
        SpotId::Giguna__Carnelian__Upper_Susar,
        Item::Wall_Climb
    );
}
#[test]
fn start_Giguna__West_Caverns__East_Susar_context_giguna__west_caverns__ctx__east_susar_True_can_obtain_Wall_Climb(
) {
    let (mut world, mut ctx) = shared_setup();
    ctx.cbits1
        .insert(flags::ContextBits1::GIGUNA__WEST_CAVERNS__CTX__EAST_SUSAR);

    expect_obtainable!(
        &world,
        ctx,
        SpotId::Giguna__West_Caverns__East_Susar,
        Item::Wall_Climb
    );
}
#[test]
fn start_Giguna__Giguna_Northeast__Save_Point_can_activate_Giguna__Carnelian__Switch__Open_Door() {
    let (mut world, mut ctx) = shared_setup();

    expect_action_accessible!(
        &world,
        ctx,
        SpotId::Giguna__Giguna_Northeast__Save_Point,
        ActionId::Giguna__Carnelian__Switch__Open_Door
    );
}
#[test]
fn start_Giguna__Carnelian__Switch_context_giguna__carnelian__ctx__door_opened_True_can_access_Giguna__Carnelian__Vault__Item(
) {
    let (mut world, mut ctx) = shared_setup();
    ctx.cbits1
        .insert(flags::ContextBits1::GIGUNA__CARNELIAN__CTX__DOOR_OPENED);

    expect_accessible!(
        &world,
        ctx,
        SpotId::Giguna__Carnelian__Switch,
        LocationId::Giguna__Carnelian__Vault__Item
    );
}
#[test]
fn start_Giguna__Giguna_Base__Save_Point_can_access_Giguna__Building_Interior__Bookshelf__Note() {
    let (mut world, mut ctx) = shared_setup();

    expect_accessible!(
        &world,
        ctx,
        SpotId::Giguna__Giguna_Base__Save_Point,
        LocationId::Giguna__Building_Interior__Bookshelf__Note
    );
}