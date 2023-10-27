use crate::condense::CondensedEdge;
use crate::context::{ContextWrapper, Ctx};
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
    fn can_access(&self, ctx: &Self::Context) -> bool;
    fn time(&self) -> u32;
    fn price(&self) -> &Self::Currency;
    fn is_free(&self) -> bool {
        *self.price() == Self::Currency::default()
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
{
}

pub trait Location: Accessible {
    type LocId: Id + enum_map::EnumArray<bool>;
    type CanonId: Id + Default;
    type ExitId: Id;

    fn id(&self) -> Self::LocId;
    fn item(&self) -> <Self::Context as Ctx>::ItemId;
    fn canon_id(&self) -> Self::CanonId;
    fn exit_id(&self) -> &Option<Self::ExitId>;
}

pub trait Exit: Accessible {
    type ExitId: Id;
    type SpotId: Id + Default + enum_map::EnumArray<Option<ContextWrapper<Self::Context>>>;
    type LocId: Id + enum_map::EnumArray<bool>;

    fn id(&self) -> Self::ExitId;
    fn dest(&self) -> Self::SpotId;
    fn connect(&mut self, dest: Self::SpotId);
    fn loc_id(&self) -> &Option<Self::LocId>;
    fn always(id: Self::ExitId) -> bool;
}

pub trait Action: Accessible {
    type ActionId: Id;
    type SpotId: Id + Default;
    fn id(&self) -> Self::ActionId;
    fn perform(&self, ctx: &mut Self::Context);
    fn dest(&self, ctx: &Self::Context) -> Self::SpotId;
}

pub trait Warp: Accessible {
    type WarpId: Id;
    type SpotId: Id + Default;

    fn id(&self) -> Self::WarpId;
    fn dest(&self, ctx: &Self::Context) -> Self::SpotId;
    fn connect(&mut self, dest: Self::SpotId);
    fn prewarp(&self, ctx: &mut Self::Context);
    fn postwarp(&self, ctx: &mut Self::Context);
    fn should_reload(&self) -> bool;
}

pub trait World: Sync + Default {
    type Location: Location;
    type Exit: Exit<
        ExitId = <Self::Location as Location>::ExitId,
        LocId = <Self::Location as Location>::LocId,
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
    const NUM_LOCATIONS: u32;

    fn objective_name(&self) -> String;
    fn get_location(&self, loc_id: <Self::Location as Location>::LocId) -> &Self::Location;
    fn get_exit(&self, ex_id: <Self::Exit as Exit>::ExitId) -> &Self::Exit;
    fn get_action(&self, act_id: <Self::Action as Action>::ActionId) -> &Self::Action;
    fn get_warp(&self, warp_id: <Self::Warp as Warp>::WarpId) -> &Self::Warp;
    fn get_canon_locations(
        &self,
        loc_id: <Self::Location as Location>::LocId,
    ) -> Vec<<Self::Location as Location>::LocId>;
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
    fn get_exit_spot(
        &self,
        exit_id: <Self::Exit as Exit>::ExitId,
    ) -> <Self::Exit as Exit>::SpotId;
    fn is_global_action(&self, act_id: <Self::Action as Action>::ActionId) -> bool {
        self.get_action_spot(act_id) == <Self::Exit as Exit>::SpotId::default()
    }

    fn skip_unused_items(&self, ctx: &mut <Self::Location as Accessible>::Context);
    fn won(&self, ctx: &<Self::Location as Accessible>::Context) -> bool;
    fn items_needed(
        &self,
        ctx: &<Self::Location as Accessible>::Context,
    ) -> Vec<(
        <<Self::Location as Accessible>::Context as Ctx>::ItemId,
        i16,
    )>;
    fn objective_items(
        &self,
    ) -> Vec<(
        <<Self::Location as Accessible>::Context as Ctx>::ItemId,
        i16,
    )>;

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
    fn spot_of_interest(&self, sp: <Self::Exit as Exit>::SpotId) -> bool;

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
