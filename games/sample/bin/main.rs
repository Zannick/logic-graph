#![allow(dead_code)]
#![allow(unused)]

extern crate enum_map;

use analyzer::access::*;
use analyzer::algo::*;
use analyzer::context::*;
use analyzer::greedy::*;
use libsample::*;

fn main() {
    let world = graph::World::new();
    let context = context::Context::new();
    if !can_win(&world, &context) {
        panic!("Cannot win on default settings");
    }
    search(&world, context);
}
