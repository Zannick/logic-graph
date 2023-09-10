use analyzer::unittest::*;
use libaxiom_verge2::context::Context;
use libaxiom_verge2::graph;
use std::path::PathBuf;

fn main() {
    let mut dir = PathBuf::from(file!());
    dir.pop();
    run_all_tests_in_dir::<graph::World, Context>(&dir);
}