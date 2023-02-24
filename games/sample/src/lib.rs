#![allow(dead_code)]

extern crate enum_map;
extern crate yaml_rust;

pub mod context;
pub mod graph;
pub mod helpers;
pub mod items;
pub mod movements;
pub mod prices;
mod rules;

use std::fs::File;
use std::io::Read;
use analyzer::settings::*;
use yaml_rust::{Yaml, YamlLoader};

fn read_key_value(
    world: &mut graph::World,
    ctx: &mut context::Context,
    key: &Yaml,
    val: &Yaml,
) -> Result<(), String> {
    match key.as_str() {
        Some("objective") => {
            world.objective = parse_str_into(key, val)?;
        }
        Some("logic_deku_b1_skip") => {
            ctx.logic_deku_b1_skip = parse_bool(key, val)?;
        }
        _ => {
            return Err(format!("Unrecognized or unparseable key: '{:?}'", key));
        }
    }
    Ok(())
}

pub fn load_settings(filename: Option<&str>) -> (graph::World, context::Context) {
    let mut world = graph::World::new();
    let mut ctx = context::Context::new();
    if let Some(filename) = filename {
        let mut file = File::open(filename).expect("Couldn't open file");
        let mut settings = String::new();
        file.read_to_string(&mut settings)
            .expect("Couldn't read from file");
        let yaml = YamlLoader::load_from_str(&settings).expect("YAML parse error");
        let mut errs = Vec::new();
        for (key, value) in yaml[0]
            .as_hash()
            .expect("YAML file should be a key-value map")
        {
            if let Err(e) = read_key_value(&mut world, &mut ctx, key, value) {
                errs.push(e);
            }
        }
    }
    (world, ctx)
}
