//! AUTOGENERATED FOR Axiom Verge 2 - MODIFICATIONS WILL BE LOST

use analyzer::algo::search;
use libaxiom_verge2::*;
use std::env;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    let (world, context) =
        settings::load_settings(if args.len() > 1 { Some(&args[1]) } else { None });
    search(&world, context)
}
