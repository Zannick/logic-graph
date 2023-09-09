#![allow(unused)]

use analyzer::context::{ContextWrapper, Ctx, History, Wrapper};
use analyzer::unittest::*;
use analyzer::world::*;
use analyzer::*;
use libaxiom_verge2::context::Context;
use libaxiom_verge2::graph::{self, *};
use libaxiom_verge2::items::Item;
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