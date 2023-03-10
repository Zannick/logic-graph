//! AUTOGENERATED FOR sample - MODIFICATIONS WILL BE LOST

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use crate::context::*;
use crate::graph::*;
use crate::items::Item;
use crate::prices::Currency;
use crate::*;
use analyzer::context::Ctx;
use analyzer::world;

pub fn access_default(_ctx: &Context) -> bool {
    true
}

pub fn access___nuts_or_can_use__slingshot_and_can_jumpslash(ctx: &Context) -> bool {
    // ($Nuts or $can_use(Slingshot)) and $can_jumpslash
    ((helper__Nuts!(ctx) || helper__can_use!(ctx, Item::Slingshot)) && helper__can_jumpslash!(ctx))
}
pub fn access_all_region_checks__deku_tree(ctx: &Context) -> bool {
    // $all_region_checks(`Deku Tree`)
    ctx.all_region_checks(RegionId::Deku_Tree)
}
pub fn access_can_play__minuet_of_forest(ctx: &Context) -> bool {
    // $can_play(Minuet_of_Forest)
    helper__can_play!(ctx, Item::Minuet_of_Forest)
}
pub fn access_can_use__boomerang_or_can_use__hookshot(ctx: &Context) -> bool {
    // $can_use(Boomerang) or $can_use(Hookshot)
    (helper__can_use!(ctx, Item::Boomerang) || helper__can_use!(ctx, Item::Hookshot))
}
pub fn access_can_use__slingshot(ctx: &Context) -> bool {
    // $can_use(Slingshot)
    helper__can_use!(ctx, Item::Slingshot)
}
pub fn access_defeat_ganon(ctx: &Context) -> bool {
    // Defeat_Ganon
    ctx.has(Item::Defeat_Ganon)
}
pub fn access_defeat_gohma(ctx: &Context) -> bool {
    // Defeat_Gohma
    ctx.has(Item::Defeat_Gohma)
}
pub fn access_deku_back_room_web_and_can_blast_or_smash(ctx: &Context) -> bool {
    // Deku_Back_Room_Web and $can_blast_or_smash
    (ctx.has(Item::Deku_Back_Room_Web) && helper__can_blast_or_smash!(ctx))
}
pub fn access_deku_back_room_web_and_deku_back_room_wall(ctx: &Context) -> bool {
    // Deku_Back_Room_Web and Deku_Back_Room_Wall
    (ctx.has(Item::Deku_Back_Room_Web) && ctx.has(Item::Deku_Back_Room_Wall))
}
pub fn access_deku_basement_block_and_is_child_and_sticks(ctx: &Context) -> bool {
    // Deku_Basement_Block and $is_child and $Sticks
    ((ctx.has(Item::Deku_Basement_Block) && helper__is_child!(ctx)) && helper__Sticks!(ctx))
}
pub fn access_deku_basement_scrubs(ctx: &Context) -> bool {
    // Deku_Basement_Scrubs
    ctx.has(Item::Deku_Basement_Scrubs)
}
pub fn access_deku_basement_switch(ctx: &Context) -> bool {
    // Deku_Basement_Switch
    ctx.has(Item::Deku_Basement_Switch)
}
pub fn access_deku_basement_web(ctx: &Context) -> bool {
    // Deku_Basement_Web
    ctx.has(Item::Deku_Basement_Web)
}
pub fn access_deku_lobby_web(ctx: &Context) -> bool {
    // Deku_Lobby_Web
    ctx.has(Item::Deku_Lobby_Web)
}
pub fn access_deku_lobby_web_and_logic_deku_b1_skip(ctx: &Context) -> bool {
    // Deku_Lobby_Web and logic_deku_b1_skip
    (ctx.has(Item::Deku_Lobby_Web) && ctx.logic_deku_b1_skip)
}
pub fn access_deku_slingshot_scrub(ctx: &Context) -> bool {
    // Deku_Slingshot_Scrub
    ctx.has(Item::Deku_Slingshot_Scrub)
}
pub fn access_deku_tree__compass_room__entry___light_torch_req(ctx: &Context) -> bool {
    // $is_child and $Sticks and not ^_torch
    ((helper__is_child!(ctx) && helper__Sticks!(ctx)) && !ctx.deku_tree__compass_room__ctx__torch())
}
pub fn access_deku_tree__compass_room__entry__floor_3_(ctx: &Context) -> bool {
    // ^_torch
    ctx.deku_tree__compass_room__ctx__torch()
}
pub fn access_false(ctx: &Context) -> bool {
    // False
    false
}
pub fn access_gold_skulltula_token__10(ctx: &Context) -> bool {
    // Gold_Skulltula_Token{10}
    ctx.count(Item::Gold_Skulltula_Token) >= 10
}
pub fn access_has_fire_source(ctx: &Context) -> bool {
    // $has_fire_source
    helper__has_fire_source!(ctx)
}
pub fn access_has_fire_source_with_torch_or_can_use__bow(ctx: &Context) -> bool {
    // $has_fire_source_with_torch or $can_use(Bow)
    (helper__has_fire_source_with_torch!(ctx) || helper__can_use!(ctx, Item::Bow))
}
pub fn access_has_shield(ctx: &Context) -> bool {
    // $has_shield
    helper__has_shield!(ctx)
}
pub fn access_is_adult(ctx: &Context) -> bool {
    // $is_adult
    helper__is_adult!(ctx)
}
pub fn access_is_adult_or_can_child_attack(ctx: &Context) -> bool {
    // $is_adult or $can_child_attack
    (helper__is_adult!(ctx) || helper__can_child_attack!(ctx))
}
pub fn access_is_adult_or_can_child_attack_or_nuts(ctx: &Context) -> bool {
    // $is_adult or $can_child_attack or $Nuts
    ((helper__is_adult!(ctx) || helper__can_child_attack!(ctx)) || helper__Nuts!(ctx))
}
pub fn access_is_adult_or_deku_basement_block(ctx: &Context) -> bool {
    // $is_adult or Deku_Basement_Block
    (helper__is_adult!(ctx) || ctx.has(Item::Deku_Basement_Block))
}
pub fn access_is_adult_or_kokiri_sword_or_boomerang(ctx: &Context) -> bool {
    // $is_adult or Kokiri_Sword or Boomerang
    ((helper__is_adult!(ctx) || ctx.has(Item::Kokiri_Sword)) || ctx.has(Item::Boomerang))
}
pub fn access_is_adult_or_showed_mido(ctx: &Context) -> bool {
    // $is_adult or Showed_Mido
    (helper__is_adult!(ctx) || ctx.has(Item::Showed_Mido))
}
pub fn access_is_adult_or_slingshot_or_sticks_or_kokiri_sword(ctx: &Context) -> bool {
    // $is_adult or Slingshot or $Sticks or Kokiri_Sword
    (((helper__is_adult!(ctx) || ctx.has(Item::Slingshot)) || helper__Sticks!(ctx))
        || ctx.has(Item::Kokiri_Sword))
}
pub fn access_is_adult_or_sticks_or_kokiri_sword(ctx: &Context) -> bool {
    // $is_adult or $Sticks or Kokiri_Sword
    ((helper__is_adult!(ctx) || helper__Sticks!(ctx)) || ctx.has(Item::Kokiri_Sword))
}
pub fn access_is_child(ctx: &Context) -> bool {
    // $is_child
    helper__is_child!(ctx)
}
pub fn access_is_child_and_kokiri_sword_and_deku_shield(ctx: &Context) -> bool {
    // $is_child and Kokiri_Sword and $Deku_Shield
    ((helper__is_child!(ctx) && ctx.has(Item::Kokiri_Sword)) && helper__Deku_Shield!(ctx))
}
pub fn access_is_child_and_sticks_and_nuts(ctx: &Context) -> bool {
    // $is_child and $Sticks and $Nuts
    ((helper__is_child!(ctx) && helper__Sticks!(ctx)) && helper__Nuts!(ctx))
}
pub fn access_kokiri_emerald(ctx: &Context) -> bool {
    // Kokiri_Emerald
    ctx.has(Item::Kokiri_Emerald)
}
pub fn access_nuts_and_has_shield_and_if___is_child____sticks__else____biggoron_sword_(
    ctx: &Context,
) -> bool {
    // $Nuts and $has_shield and if ($is_child) { $Sticks } else { Biggoron_Sword }
    ((helper__Nuts!(ctx) && helper__has_shield!(ctx))
        && if helper__is_child!(ctx) {
            helper__Sticks!(ctx)
        } else {
            ctx.has(Item::Biggoron_Sword)
        })
}
pub fn access_triforce_piece__triforce_count(ctx: &Context) -> bool {
    // Triforce_Piece{triforce_count}
    ctx.count(Item::Triforce_Piece) >= ctx.triforce_count
}
pub fn action_deku_tree__compass_room__entry___light_torch__do(ctx: &mut Context) {
    // ^_torch = True
    ctx.deku_tree__compass_room__ctx__torch = true;
}
pub fn action_has_effect_deku_tree__compass_room__entry___light_torch__do(ctx: &Context) -> bool {
    ctx.deku_tree__compass_room__ctx__torch != true
}
pub fn action_rupees__max__rupees__20_wallet_max(ctx: &mut Context) {
    // ^rupees = $max(^rupees + 20, $wallet_max)
    ctx.rupees = std::cmp::max(ctx.rupees() + 20, helper__wallet_max!(ctx));
}
pub fn action_has_effect_rupees__max__rupees__20_wallet_max(ctx: &Context) -> bool {
    ctx.rupees != std::cmp::max(ctx.rupees() + 20, helper__wallet_max!(ctx))
}
pub fn action_rupees__min__rupees__1_wallet_max(ctx: &mut Context) {
    // ^rupees = $min(^rupees + 1, $wallet_max)
    ctx.rupees = std::cmp::min(ctx.rupees() + 1, helper__wallet_max!(ctx));
}
pub fn action_has_effect_rupees__min__rupees__1_wallet_max(ctx: &Context) -> bool {
    ctx.rupees != std::cmp::min(ctx.rupees() + 1, helper__wallet_max!(ctx))
}
pub fn action_rupees__min__rupees__50_wallet_max(ctx: &mut Context) {
    // ^rupees = $min(^rupees + 50, $wallet_max)
    ctx.rupees = std::cmp::min(ctx.rupees() + 50, helper__wallet_max!(ctx));
}
pub fn action_has_effect_rupees__min__rupees__50_wallet_max(ctx: &Context) -> bool {
    ctx.rupees != std::cmp::min(ctx.rupees() + 50, helper__wallet_max!(ctx))
}
pub fn action_rupees__min__rupees__5_wallet_max(ctx: &mut Context) {
    // ^rupees = $min(^rupees + 5, $wallet_max)
    ctx.rupees = std::cmp::min(ctx.rupees() + 5, helper__wallet_max!(ctx));
}
pub fn action_has_effect_rupees__min__rupees__5_wallet_max(ctx: &Context) -> bool {
    ctx.rupees != std::cmp::min(ctx.rupees() + 5, helper__wallet_max!(ctx))
}
pub fn action_save__position(ctx: &mut Context) {
    // ^save = ^position
    ctx.save = ctx.position();
}
pub fn action_has_effect_save__position(ctx: &Context) -> bool {
    ctx.save != ctx.position()
}
pub fn action_tod__match_tod____day__night_night__day____day_(ctx: &mut Context) {
    // ^tod = MATCH ^tod { 'day' => 'night', 'night' => 'day', _ => 'day' }
    ctx.tod = match ctx.tod() {
        enums::Tod::Day => enums::Tod::Night,
        enums::Tod::Night => enums::Tod::Day,
        _ => enums::Tod::Day,
    };
}
pub fn action_has_effect_tod__match_tod____day__night_night__day____day_(ctx: &Context) -> bool {
    ctx.tod
        != match ctx.tod() {
            enums::Tod::Day => enums::Tod::Night,
            enums::Tod::Night => enums::Tod::Day,
            _ => enums::Tod::Day,
        }
}
