#![allow(dead_code)]

extern crate enum_map;

mod context;
mod graph;
mod helpers;
mod items;

use analyzer::context::ItemContext;
use analyzer::world;
use context::Context;
use enum_map::EnumMap;
use items::Item;

#[derive(Copy, Clone)]
struct Location {
    id: graph::Location,
    item: Item,
    // Collecting from a location with a specific non-None canonical value
    // shall clear the item from all such Locations
    canonical: graph::Canon,
    time: i8,
    price: i8,

    access_rule: fn(&Context) -> bool,
}

impl world::Location for Location {
    type LocId = graph::Location;
    type CanonId = graph::Canon;
    type ItemContext = Context;

    fn id(&self) -> &graph::Location {
        &self.id
    }

    fn get_item(&self) -> &Item {
        &self.item
    }

    fn clear_item(&mut self) {
        self.item = Item::None;
    }

    fn get_canon_id(&self) -> &graph::Canon {
        &self.canonical
    }

    fn can_access(&self, ctx: &Context) -> bool {
        (self.access_rule)(ctx)
    }

    fn take(&mut self, ctx: &mut Context) {
        ctx.collect(&self.item);
        self.clear_item();
    }
}

#[derive(Copy, Clone)]
struct Exit {
    id: graph::Exit,
    time: i8,
    dest: graph::Spot,
    access_rule: fn(&Context) -> bool,
}

impl world::Exit for Exit {
    type ExitId = graph::Exit;
    type SpotId = graph::Spot;
    type ItemContext = Context;

    fn id(&self) -> &graph::Exit {
        &self.id
    }

    fn dest(&self) -> &graph::Spot {
        &self.dest
    }

    fn connect(&mut self, dest: &graph::Spot) {
        self.dest = *dest;
    }

    fn can_access(&self, ctx: &Context) -> bool {
        (self.access_rule)(ctx)
    }
}

#[derive(Copy, Clone)]
struct Hybrid {
    id: graph::Exit,
    item: Item,
    canonical: graph::Canon,
    time: i8,
}

#[derive(Copy, Clone)]
struct Spot<'a> {
    id: graph::Spot,
    // we can hold slices here to the real things held in World
    locations: &'a [Location],
    exits: &'a [Exit],
    hybrids: &'a [Hybrid],
}

#[derive(Copy, Clone)]
struct World<'a> {
    state: Context,
    // These are arrays that group the items together by their parent.
    // Using EnumMap for this ONLY WORKS if the keys are properly ordered to group
    // nearby things together.
    // For entrance rando, we would need to have a layer of indirection:
    // list_index: EnumMap<EnumType, usize>,
    // list: EnumArray<ObjType>,
    locations: EnumMap<graph::Location, Location>,
    exits: EnumMap<graph::Exit, Exit>,
    spots: EnumMap<graph::Spot, Spot<'a>>,
}

impl<'a> world::World for World<'a> {
    type Location = Location;
    type Exit = Exit;
    type Spot = Spot<'a>;
    type ItemContext = Context;

    fn get_location(&self, locid: &graph::Location) -> &Location {
        &self.locations[*locid]
    }
    fn get_location_mut(&mut self, locid: &graph::Location) -> &mut Location {
        &mut self.locations[*locid]
    }
}

fn main() {
    println!("Hello, world!");
    helper__Nuts!();
    helper__can_use!("Slingshot");
    helper___is_child_item!("Slingshot");
    println!("Slingshot = {:?}", Item::Slingshot);
    let x = graph::Location::KF__Shop__Entry__Blue_Rupee;
    println!("{:?} = {}", x, x);
}
