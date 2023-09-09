#![allow(unused)]

use analyzer::context::{ContextWrapper, Ctx, History, Wrapper};
use analyzer::unittest::*;
use analyzer::world::*;
use analyzer::*;
use libaxiom_verge2::context::Context;
use libaxiom_verge2::graph::{self, *};
use libaxiom_verge2::items::Item;
use std::fs;
use std::path::PathBuf;
use yaml_rust::*;

#[test]
fn test() {
    let tc = "name: Obtain\n\
              with:\n\
                - Switch_36_11\n\
              can_obtain: Ledge_Grab";
    let yaml = YamlLoader::load_from_str(&tc).expect("YAML parse error");

    let mut world = graph::World::new();
    world.condense_graph();
    let mut ctx = Context::default();
    let test = build_test(&yaml[0], &ctx, "test case 0").unwrap();
    assert!(matches!(test.mode, TestMode::Obtainable(true, Item::Ledge_Grab)));
}

#[test]
fn print_test() {
    let mut dir = PathBuf::from(file!());
    dir.pop();

    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let ext = path.extension().map(|s| s.to_str()).flatten();
        if matches!(ext, Some("yaml")) {
            run_test_file::<graph::World, Context>(&path);
        }
    }
}