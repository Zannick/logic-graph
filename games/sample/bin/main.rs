#![allow(dead_code)]

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
    let ctx = ContextWrapper::new(context);
    if let Some(ctx) = greedy_search(&world, &ctx) {
        println!("Found greedy solution of {}ms.", ctx.elapsed());
        let fresh = context::Context::new();
        let m = minimize_playthrough(&world, &fresh, &ctx);

        println!("Minimized to {}ms:", m.elapsed());
        //println!("{}", m.history_str());
    } else {
        println!("Did not find a solution");
    }
}
