use crate::condense::CondensedEdge;
use crate::context::Ctx;
use crate::new_hashset;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::option::Option;

// type graph
// Context
// -> World
//    -> Location -> LocId, ExitId, CanonId
//       Accessible -> Context -> ItemId
//    -> Exit      --^    --^   -> SpotId
//       Accessible -> Context -> ItemId
//    -> Action -> ActionId
//       Accessible -> Context -> ItemId

pub trait Accessible: Sync {
    type Context: Ctx;
    type Currency: Id + Default;
    fn can_access(&self, ctx: &Self::Context, world: &<Self::Context as Ctx>::World) -> bool;
    fn observe_access(
        &self,
        ctx: &Self::Context,
        world: &<Self::Context as Ctx>::World,
        observer: &mut <Self::Context as Ctx>::Observer,
    ) -> bool;
    /// Base access time, assumed to be a constant.
    fn base_time(&self) -> u32;
    /// Actual access time. Must be at least base_time.
    fn time(&self, ctx: &Self::Context, world: &<Self::Context as Ctx>::World) -> u32;
    fn base_price(&self) -> &Self::Currency;
    fn price(&self, ctx: &Self::Context, world: &<Self::Context as Ctx>::World) -> Self::Currency;
    fn is_free(&self) -> bool {
        *self.base_price() == Self::Currency::default()
    }

    fn explain_rule(
        &self,
        ctx: &Self::Context,
        world: &<Self::Context as Ctx>::World,
        edict: &mut FxHashMap<&'static str, String>,
    ) -> (bool, Vec<&'static str>);
    fn explain(&self, ctx: &Self::Context, world: &<Self::Context as Ctx>::World) -> String
    where
        <<Self::Context as Ctx>::World as World>::Location: Accessible<Currency = Self::Currency>,
    {
        let mut edict = FxHashMap::default();
        let mut explains = Vec::new();
        if !ctx.can_afford(self.base_price()) {
            explains.push(format!(
                "Can't afford {}: have {}",
                self.base_price(),
                ctx.amount_could_afford(self.base_price())
            ));
        }
        let (_, named) = self.explain_rule(ctx, world, &mut edict);
        // Only display each once.
        let mut seen = new_hashset();
        for key in named {
            if !seen.contains(&key) {
                assert!(
                    edict.contains_key(&key),
                    "Attempting to explain {} but it's not in edict",
                    key
                );
                explains.push(format!("{}: {}", key, edict[&key]));
                seen.insert(key);
            }
        }
        explains.join("\n")
    }
}

pub trait Id:
    Copy
    + Clone
    + Debug
    + Display
    + Eq
    + std::str::FromStr<Err = String>
    + Hash
    + Ord
    + PartialOrd
    + PartialEq
    + Display
    + Send
    + Sync
    + Serialize
    + for<'a> Deserialize<'a>
    + 'static
{
}

pub trait Location: Accessible {
    type LocId: Id + enum_map::EnumArray<bool>;
    type CanonId: Id;
    type SpotId: Id + Default;

    fn id(&self) -> Self::LocId;
    fn item(&self) -> <Self::Context as Ctx>::ItemId;
    fn canon_id(&self) -> Self::CanonId;
    fn skippable(&self) -> bool;

    fn dest(&self) -> Self::SpotId {
        Default::default()
    }
}

pub trait Exit: Accessible {
    type ExitId: Id;
    type SpotId: Id + Default;

    fn id(&self) -> Self::ExitId;
    fn dest(&self) -> Self::SpotId;
    fn connect(&mut self, dest: Self::SpotId);
    fn always(id: Self::ExitId) -> bool;
    fn has_penalties(id: Self::ExitId) -> bool;
}

pub trait Action: Accessible {
    type ActionId: Id;
    type SpotId: Id + Default;
    fn id(&self) -> Self::ActionId;
    fn perform(&self, ctx: &mut Self::Context, world: &<Self::Context as Ctx>::World);
    fn dest(&self, ctx: &Self::Context, world: &<Self::Context as Ctx>::World) -> Self::SpotId;
    fn observe_effects(
        &self,
        ctx: &Self::Context,
        world: &<Self::Context as Ctx>::World,
        observer: &mut <Self::Context as Ctx>::Observer,
    );
}

pub trait Warp: Accessible {
    type WarpId: Id;
    type SpotId: Id + Default;

    fn id(&self) -> Self::WarpId;
    fn dest(&self, ctx: &Self::Context, world: &<Self::Context as Ctx>::World) -> Self::SpotId;
    fn connect(&mut self, dest: Self::SpotId);
    fn prewarp(&self, ctx: &mut Self::Context, world: &<Self::Context as Ctx>::World);
    fn postwarp(&self, ctx: &mut Self::Context, world: &<Self::Context as Ctx>::World);
    fn should_reload(&self) -> bool;
    fn is_bulk_exit(&self) -> bool;
    fn observe_effects(
        &self,
        ctx: &Self::Context,
        world: &<Self::Context as Ctx>::World,
        observer: &mut <Self::Context as Ctx>::Observer,
    );
}

pub trait World: Sync {
    type Location: Location;
    type Exit: Exit<
        SpotId = <Self::Location as Location>::SpotId,
        Context = <Self::Location as Accessible>::Context,
        Currency = <Self::Location as Accessible>::Currency,
    >;
    type Action: Action<
        Context = <Self::Location as Accessible>::Context,
        SpotId = <Self::Exit as Exit>::SpotId,
        Currency = <Self::Location as Accessible>::Currency,
    >;
    type Warp: Warp<
        Context = <Self::Location as Accessible>::Context,
        SpotId = <Self::Exit as Exit>::SpotId,
        Currency = <Self::Location as Accessible>::Currency,
    >;
    const NUM_CANON_LOCATIONS: usize;

    fn new() -> Box<Self>;

    fn ruleset(&self) -> String;
    fn get_location(&self, loc_id: <Self::Location as Location>::LocId) -> &Self::Location;
    fn get_exit(&self, ex_id: <Self::Exit as Exit>::ExitId) -> &Self::Exit;
    fn get_action(&self, act_id: <Self::Action as Action>::ActionId) -> &Self::Action;
    fn get_warp(&self, warp_id: <Self::Warp as Warp>::WarpId) -> &Self::Warp;
    fn get_item_locations(
        &self,
        item_id: <<Self::Location as Accessible>::Context as Ctx>::ItemId,
    ) -> Vec<<Self::Location as Location>::LocId>;

    fn get_spot_locations(&self, spot_id: <Self::Exit as Exit>::SpotId) -> &[Self::Location];
    fn get_spot_exits(&self, spot_id: <Self::Exit as Exit>::SpotId) -> &[Self::Exit];
    fn get_spot_actions(&self, spot_id: <Self::Exit as Exit>::SpotId) -> &[Self::Action];
    fn get_global_actions(&self) -> &[Self::Action];
    fn get_all_spots(&self) -> &[<Self::Exit as Exit>::SpotId];
    fn same_region(sp1: <Self::Exit as Exit>::SpotId, sp2: <Self::Exit as Exit>::SpotId) -> bool;
    fn same_area(sp1: <Self::Exit as Exit>::SpotId, sp2: <Self::Exit as Exit>::SpotId) -> bool;

    fn get_area_spots(
        &self,
        spot_id: <Self::Exit as Exit>::SpotId,
    ) -> &[<Self::Exit as Exit>::SpotId];
    fn get_warps(&self) -> &[Self::Warp];
    fn get_all_locations(&self) -> &[Self::Location];
    fn get_location_spot(
        &self,
        loc_id: <Self::Location as Location>::LocId,
    ) -> <Self::Exit as Exit>::SpotId;
    fn get_action_spot(
        &self,
        act_id: <Self::Action as Action>::ActionId,
    ) -> <Self::Exit as Exit>::SpotId;
    fn get_exit_spot(&self, exit_id: <Self::Exit as Exit>::ExitId) -> <Self::Exit as Exit>::SpotId;
    fn is_global_action(&self, act_id: <Self::Action as Action>::ActionId) -> bool {
        self.get_action_spot(act_id) == <Self::Exit as Exit>::SpotId::default()
    }
    fn action_has_visit(act_id: <Self::Action as Action>::ActionId) -> bool;

    fn won(&self, ctx: &<Self::Location as Accessible>::Context) -> bool;
    fn items_needed(
        &self,
        ctx: &<Self::Location as Accessible>::Context,
    ) -> Vec<(
        <<Self::Location as Accessible>::Context as Ctx>::ItemId,
        i16,
    )>;
    fn required_items(
        &self,
    ) -> Vec<(
        <<Self::Location as Accessible>::Context as Ctx>::ItemId,
        i16,
    )>;
    fn unused_items(&self) -> Vec<<<Self::Location as Accessible>::Context as Ctx>::ItemId>;
    fn remaining_items(
        &self,
        ctx: &<Self::Location as Accessible>::Context,
    ) -> Vec<(
        <<Self::Location as Accessible>::Context as Ctx>::ItemId,
        i16,
    )>;

    fn should_draw_edge(&self, exit_id: <Self::Exit as Exit>::ExitId) -> bool;
    fn should_draw_spot(&self, spot_id: <Self::Exit as Exit>::SpotId) -> bool;

    /// Edge connections for the purpose of Steiner graph.
    fn base_edges(
        &self,
    ) -> Vec<(
        <Self::Exit as Exit>::SpotId,
        <Self::Exit as Exit>::SpotId,
        u32,
    )>;

    fn are_spots_connected(
        &self,
        sp1: <Self::Exit as Exit>::SpotId,
        sp2: <Self::Exit as Exit>::SpotId,
    ) -> bool;
    fn free_movement(
        sp1: <Self::Exit as Exit>::SpotId,
        sp2: <Self::Exit as Exit>::SpotId,
    ) -> Option<u32>;
    fn best_movements(
        sp1: <Self::Exit as Exit>::SpotId,
        sp2: <Self::Exit as Exit>::SpotId,
    ) -> (
        Option<u32>,
        Vec<(
            <<Self::Location as Accessible>::Context as Ctx>::MovementState,
            u32,
        )>,
    );

    fn min_warp_time(&self) -> u32;
    /// Returns the Euclidean straight-line distance between spots' coordinates,
    /// or infinity if either has no coordinate.
    fn spot_distance(a: <Self::Exit as Exit>::SpotId, b: <Self::Exit as Exit>::SpotId) -> f32;
    fn spot_of_interest(&self, sp: <Self::Exit as Exit>::SpotId) -> bool;

    fn spot_community(spot_id: <Self::Exit as Exit>::SpotId) -> usize;
    fn location_community(loc_id: <Self::Location as Location>::LocId) -> usize;
    fn action_community(act_id: <Self::Action as Action>::ActionId) -> usize;
    fn exit_community(exit_id: <Self::Exit as Exit>::ExitId) -> usize;
    fn same_community(
        spot1: <Self::Exit as Exit>::SpotId,
        spot2: <Self::Exit as Exit>::SpotId,
    ) -> bool;
    fn get_community(
        spot: <Self::Exit as Exit>::SpotId,
    ) -> &'static FxHashSet<<Self::Exit as Exit>::SpotId>;

    fn condense_graph(&mut self);
    fn get_condensed_edges_from(
        &self,
        spot_id: <Self::Exit as Exit>::SpotId,
    ) -> &[CondensedEdge<
        <Self::Location as Accessible>::Context,
        <Self::Exit as Exit>::SpotId,
        <Self::Exit as Exit>::ExitId,
    >];
}
