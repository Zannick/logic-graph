#![allow(unused)]

use crate::context::*;
use crate::world::*;
use crate::*;
use lazy_static::lazy_static;
use libtest_mimic::*;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use yaml_rust::*;

pub enum TestMode<'a, T>
where
    T: Ctx,
{
    Obtainable(bool, T::ItemId),
    Reachable(bool, <<T::World as World>::Exit as Exit>::SpotId),
    EventuallyGets(T::ItemId),
    Route(Vec<(HistoryAlias<T>, &'a str)>),
    RequiresToObtain(T, T::ItemId),
    RequiresToReach(T, <<T::World as World>::Exit as Exit>::SpotId),
    EventuallyRequiresToObtain(T, T::ItemId, u32),
    EventuallyRequiresToReach(T, <<T::World as World>::Exit as Exit>::SpotId, u32),
}

pub struct Unittest<'a, T>
where
    T: Ctx,
{
    name: String,
    initial: T,
    mode: TestMode<'a, T>,
}

lazy_static! {
    static ref ITEM_RE: Regex = Regex::new(r"(\w+)\s*(?:\{(\d+\)})?").unwrap();
}

fn item_from_yaml<T>(yaml: &Yaml) -> anyhow::Result<(T::ItemId, u32), String>
where
    T: Ctx,
{
    if let Some(s) = yaml.as_str() {
        if let Some(caps) = ITEM_RE.captures(s) {
            let item = T::ItemId::from_str(&caps[1])?;
            let ct = if caps[2].is_empty() {
                1
            } else {
                u32::from_str(&caps[2]).map_err(|e| format!("{:?}", e))?
            };
            Ok((item, ct))
        } else {
            Err(format!("Value did not parse: {:?}", yaml))
        }
    } else {
        Err(format!("Item value not a string: {:?}", yaml))
    }
}

fn handle_with<T>(yaml: &Yaml, ctx: &mut T, name: &str, errs: &mut Vec<String>)
where
    T: Ctx,
{
    if let Some(list) = yaml.as_vec() {
        for istr in list {
            match item_from_yaml::<T>(istr) {
                Ok((item, ct)) => {
                    for _i in 0..ct {
                        ctx.add_item(item);
                    }
                }
                Err(e) => {
                    errs.push(format!("{}.{}: {}", name, "with", e));
                }
            }
        }
    } else {
        errs.push(format!(
            "{}.{}: Value is not list: {:?}",
            name, "with", yaml
        ));
    }
}

fn handle_context_values<T>(ctx: &mut T, yaml: &Yaml, name: &str, errs: &mut Vec<String>)
where
    T: Ctx,
{
    if let Some(map) = yaml.as_hash() {
        for (key, value) in map {
            let key = match key.as_str() {
                Some(k) => k,
                _ => {
                    errs.push(format!(
                        "{}.{}: Expected str key: {:?}",
                        name, "context", key
                    ));
                    continue;
                }
            };
           
            if let Err(e) = ctx.parse_set_context(key, value) {
                errs.push(format!("{}.{}: {:?}: {}", name, "context", key, e));
            }
        }
    } else {
        errs.push(format!(
            "{}.{}: Value is not map: {:?}",
            name, "context", yaml
        ));
    }
}

pub fn apply_context<W, T>(world: &W, ctx: &mut T, yaml: &Yaml, name: &str, errs: &mut Vec<String>)
where
    W: World,
    T: Ctx<World = W>,
{
    let map = if let Some(map) = yaml.as_hash() {
        map
    } else {
        errs.push(format!("{}: Expected key-value map", name));
        return;
    };

    for (key, value) in map {
        match key.as_str() {
            Some("with") => {
                handle_with(value, ctx, name, errs);
            }
            Some("context") => {
                handle_context_values(ctx, yaml, name, errs);
            }
            _ => {
                errs.push(format!("{}: Unrecognized key {:?}", name, key));
            }
        }
    }
}

pub fn run_test_file<W, T>(filename: &PathBuf)
where
    T: Ctx<World = W>,
    W: World,
{
    let mut file = File::open(filename)
        .unwrap_or_else(|e| panic!("Couldn't open file \"{:?}\": {:?}", filename, e));
    let mut settings = String::new();
    file.read_to_string(&mut settings)
        .unwrap_or_else(|e| panic!("Couldn't read from file \"{:?}\": {:?}", filename, e));
    let yaml = YamlLoader::load_from_str(&settings).expect("YAML parse error");
    let mut errs = Vec::new();
    let mut world = W::default();
    world.condense_graph();
    let mut ctx = T::default();
    let mut tests: Vec<Unittest<'_, T>> = Vec::new();

    for (key, value) in yaml[0]
        .as_hash()
        .expect("YAML file should be a key-value map")
    {
        if key.as_str() == Some("all") {
            apply_context(&world, &mut ctx, &value, "all", &mut errs);
        }
    }
}
