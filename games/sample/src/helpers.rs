//! AUTOGENERATED FOR sample - DO NOT MODIFY
//!
//! Context (game state).

/// $is_child (  )
/// ^child
#[macro_export]
macro_rules! helper__is_child {
    ($ctx:expr) => {{
        $ctx.child
    }};
}

/// $is_adult (  )
/// NOT ^child
#[macro_export]
macro_rules! helper__is_adult {
    ($ctx:expr) => {{
        !$ctx.child
    }};
}

/// $Deku_Shield (  )
/// Buy_Deku_Shield or Deku_Shield_Drop
#[macro_export]
macro_rules! helper__Deku_Shield {
    ($ctx:expr) => {{
        ($ctx.has(&Item::Buy_Deku_Shield) || $ctx.has(&Item::Deku_Shield_Drop))
    }};
}

/// $Nuts (  )
/// Buy_Deku_Nut_5 or Buy_Deku_Nut_10 or Deku_Nut_Drop
#[macro_export]
macro_rules! helper__Nuts {
    ($ctx:expr) => {{
        (($ctx.has(&Item::Buy_Deku_Nut_5) || $ctx.has(&Item::Buy_Deku_Nut_10))
            || $ctx.has(&Item::Deku_Nut_Drop))
    }};
}

/// $Sticks (  )
/// Buy_Deku_Stick_1 or Deku_Stick_Drop
#[macro_export]
macro_rules! helper__Sticks {
    ($ctx:expr) => {{
        ($ctx.has(&Item::Buy_Deku_Stick_1) || $ctx.has(&Item::Deku_Stick_Drop))
    }};
}

/// $wallet_max (  )
/// PER Progressive_Wallet { 3 => 999; 2 => 500; 1 => 200; _ => 99 }
#[macro_export]
macro_rules! helper__wallet_max {
    ($ctx:expr) => {{
        match $ctx.count(&Item::Progressive_Wallet) {
            3 => 999,
            2 => 500,
            1 => 200,
            _ => 99,
        }
    }};
}

/// $has_shield (  )
/// ($is_adult and Hylian_Shield) or ($is_child and $Deku_Shield)
#[macro_export]
macro_rules! helper__has_shield {
    ($ctx:expr) => {{
        ((helper__is_adult!($ctx) && $ctx.has(&Item::Hylian_Shield))
            || (helper__is_child!($ctx) && helper__Deku_Shield!($ctx)))
    }};
}

/// $can_play ( song )
/// Ocarina and ^song
#[macro_export]
macro_rules! helper__can_play {
    ($ctx:expr, $song:expr) => {{
        ($ctx.has(&Item::Ocarina) && $ctx.has(&$song))
    }};
}

/// $can_jumpslash (  )
/// $is_adult or $Sticks or Kokiri_Sword
#[macro_export]
macro_rules! helper__can_jumpslash {
    ($ctx:expr) => {{
        ((helper__is_adult!($ctx) || helper__Sticks!($ctx)) || $ctx.has(&Item::Kokiri_Sword))
    }};
}

/// $can_use ( item )
/// IF ($_is_magic_item(^item)) { ^item and Magic_Meter } ELSE IF ($_is_adult_item(^item)) { $is_adult and ^item } ELSE IF ($_is_magic_arrow(^item)) { $is_adult and ^item and Bow and Magic_Meter } ELSE IF ($_is_child_item(^item)) { $is_child and ^item }

#[macro_export]
macro_rules! helper__can_use {
    ($ctx:expr, $item:expr) => {{
        if helper___is_magic_item!($ctx, $item) {
            ($ctx.has(&$item) && $ctx.has(&Item::Magic_Meter))
        } else if helper___is_adult_item!($ctx, $item) {
            (helper__is_adult!($ctx) && $ctx.has(&$item))
        } else if helper___is_magic_arrow!($ctx, $item) {
            (((helper__is_adult!($ctx) && $ctx.has(&$item)) && $ctx.has(&Item::Bow))
                && $ctx.has(&Item::Magic_Meter))
        } else if helper___is_child_item!($ctx, $item) {
            (helper__is_child!($ctx) && $ctx.has(&$item))
        } else {
            false
        }
    }};
}

/// $_is_magic_item ( item )
/// ^item IN [Dins_Fire, Farores_Wind, Nayrus_Love, Lens_of_Truth]
#[macro_export]
macro_rules! helper___is_magic_item {
    ($ctx:expr, $item:expr) => {{
        match $item {
            Item::Dins_Fire | Item::Farores_Wind | Item::Nayrus_Love | Item::Lens_of_Truth => true,
            _ => false,
        }
    }};
}

/// $_is_adult_item ( item )
/// ^item IN [Bow, Megaton_Hammer, Iron_Boots, Hover_Boots, Hookshot, Goron_Tunic, Zora_Tunic, Mirror_Shield]
#[macro_export]
macro_rules! helper___is_adult_item {
    ($ctx:expr, $item:expr) => {{
        match $item {
            Item::Bow
            | Item::Megaton_Hammer
            | Item::Iron_Boots
            | Item::Hover_Boots
            | Item::Hookshot
            | Item::Goron_Tunic
            | Item::Zora_Tunic
            | Item::Mirror_Shield => true,
            _ => false,
        }
    }};
}

/// $_is_child_item ( item )
/// ^item IN [Slingshot, Boomerang, Kokiri_Sword]
#[macro_export]
macro_rules! helper___is_child_item {
    ($ctx:expr, $item:expr) => {{
        match $item {
            Item::Slingshot | Item::Boomerang | Item::Kokiri_Sword => true,
            _ => false,
        }
    }};
}

/// $_is_magic_arrow ( item )
/// ^item IN [Fire_Arrows, Light_Arrows, Blue_Fire_Arrows]
#[macro_export]
macro_rules! helper___is_magic_arrow {
    ($ctx:expr, $item:expr) => {{
        match $item {
            Item::Fire_Arrows | Item::Light_Arrows | Item::Blue_Fire_Arrows => true,
            _ => false,
        }
    }};
}

/// $has_explosives (  )
/// Bombs
#[macro_export]
macro_rules! helper__has_explosives {
    ($ctx:expr) => {{
        $ctx.has(&Item::Bombs)
    }};
}

/// $can_blast_or_smash (  )
/// $has_explosives or $can_use(Megaton_Hammer)
#[macro_export]
macro_rules! helper__can_blast_or_smash {
    ($ctx:expr) => {{
        (helper__has_explosives!($ctx) || helper__can_use!($ctx, Item::Megaton_Hammer))
    }};
}

/// $can_child_attack (  )
/// $is_child and (Slingshot or Boomerang or $Sticks or Kokiri_Sword)
#[macro_export]
macro_rules! helper__can_child_attack {
    ($ctx:expr) => {{
        (helper__is_child!($ctx)
            && ((($ctx.has(&Item::Slingshot) || $ctx.has(&Item::Boomerang))
                || helper__Sticks!($ctx))
                || $ctx.has(&Item::Kokiri_Sword)))
    }};
}

/// $has_fire_source (  )
/// $can_use(Dins_Fire) or $can_use(Fire_Arrows)
#[macro_export]
macro_rules! helper__has_fire_source {
    ($ctx:expr) => {{
        (helper__can_use!($ctx, Item::Dins_Fire) || helper__can_use!($ctx, Item::Fire_Arrows))
    }};
}

/// $has_fire_source_with_torch (  )
/// $has_fire_source or ($is_child and $Sticks)
#[macro_export]
macro_rules! helper__has_fire_source_with_torch {
    ($ctx:expr) => {{
        (helper__has_fire_source!($ctx) || (helper__is_child!($ctx) && helper__Sticks!($ctx)))
    }};
}