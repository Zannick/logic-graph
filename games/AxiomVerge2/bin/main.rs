//! AUTOGENERATED FOR Axiom Verge 2 - MODIFICATIONS WILL BE LOST


use analyzer::access::can_win;
use analyzer::algo::search;
use libaxiom_verge2::*;
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