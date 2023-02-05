#![allow(dead_code)]

extern crate enum_map;

mod context;
mod graph;
mod helpers;
mod items;

use analyzer::world;
use context::Context;
use items::Item;

trait TakesTime {
    fn time(&self) -> i8;
}

struct Location {
    id: graph::Location,
    item: Item,
    // Collecting from a location with a specific non-None canonical value
    // shall clear the item from all such Locations
    canonical: graph::Canon,
    time: i8,
}

impl world::Location<graph::Location, Item> for Location {
    fn id(&self) -> &graph::Location {
        &self.id
    }

    fn get_item(&self) -> &Item {
        &self.item
    }

    fn clear_item(&mut self) {
        self.item = Item::None;
    }
}

struct Exit {
    id: graph::Exit,
    time: i8,
}

struct Hybrid {
    id: graph::Exit,
    item: Item,
    canonical: graph::Canon,
    time: i8,
}

enum Place {
    Location(Location),
    Hybrid(Hybrid),
}

struct Node {
    id: graph::Spot,
    points: Vec<Place>,
    exits: Vec<Exit>,
}

struct World {
    state: Context,
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
