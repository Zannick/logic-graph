#![allow(dead_code)]

extern crate enum_map;

use analyzer::algo::*;
use libsample::*;

fn main() {
    let world = graph::World::new();
    let context = context::Context::new();
    do_the_thing(&world, context);
}
