#![allow(dead_code)]
#![allow(unused)]

extern crate enum_map;

use analyzer::access::*;
use analyzer::algo::*;
use analyzer::context::*;
use analyzer::greedy::*;
use libsample::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (world, context) =
        settings::load_settings(if args.len() > 1 { Some(&args[1]) } else { None });
    if !can_win(&world, &context) {
        panic!(
            "Cannot win on {} settings",
            if args.len() > 1 {
                "provided"
            } else {
                "default"
            }
        );
    }
    search(&world, context);
}
