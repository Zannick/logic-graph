#![allow(dead_code)]

extern crate enum_map;

mod context;
mod graph;
mod helpers;
mod items;
mod prices;

use items::Item;

fn main() {
    println!("Hello, world!");
    helper__Nuts!();
    helper__can_use!("Slingshot");
    helper___is_child_item!("Slingshot");
    println!("Slingshot = {:?}", Item::Slingshot);
    let x = graph::LocationId::KF__Shop__Entry__Blue_Rupee;
    println!("{:?} = {}", x, x);
}
