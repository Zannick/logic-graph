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

fn obj_from_yaml<V>(yaml: &Yaml) -> anyhow::Result<V, String>
where
    V: FromStr<Err = String>,
{
    if let Some(s) = yaml.as_str() {
        V::from_str(s)
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
                    errs.push(format!("{}: {}", name, e));
                }
            }
        }
    } else {
        errs.push(format!("{}: Value is not list: {:?}", name, yaml));
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
                    errs.push(format!("{}: Expected str key: {:?}", name, key));
                    continue;
                }
            };

            if let Err(e) = ctx.parse_set_context(key, value) {
                errs.push(format!("{}: {:?}: {}", name, key, e));
            }
        }
    } else {
        errs.push(format!("{}: Value is not map: {:?}", name, yaml));
    }
}

fn get_locations<LocId>(
    loc_list: &Yaml,
    name: &str,
    errs: &mut Vec<String>,
) -> Result<Vec<LocId>, ()>
where
    LocId: Id,
{
    let mut vec = Vec::new();
    let mut errors = false;
    if let Some(list) = loc_list.as_vec() {
        for istr in list {
            match obj_from_yaml::<LocId>(istr) {
                Ok(loc) => vec.push(loc),
                Err(e) => {
                    errors = true;
                    errs.push(format!("{}: {}", name, e))
                }
            }
        }
    } else {
        errs.push(format!("{}: Value is not list: {:?}", name, loc_list));
    }
    if errors {
        Err(())
    } else {
        Ok(vec)
    }
}

pub fn apply_test_setup<W, T>(
    ctx: &mut T,
    yaml: &Yaml,
    name: &str,
    errs: &mut Vec<String>,
    ignore_unrecognized: bool,
) where
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
        let name = format!("{}.{:?}", name, key);
        match key.as_str() {
            Some("with") => {
                handle_with(value, ctx, &name, errs);
            }
            // these should be separate, but eh
            Some("context") | Some("settings") => {
                handle_context_values(ctx, yaml, &name, errs);
            }
            Some("visited") => {
                if let Ok(locs) =
                    get_locations::<<W::Location as Location>::LocId>(value, &name, errs)
                {
                    for loc in locs {
                        ctx.reset(loc);
                        ctx.visit(loc);
                    }
                }
            }
            Some("skipped") => {
                if let Ok(locs) =
                    get_locations::<<W::Location as Location>::LocId>(value, &name, errs)
                {
                    for loc in locs {
                        ctx.reset(loc);
                        ctx.skip(loc);
                    }
                }
            }
            Some("start") => match obj_from_yaml::<<W::Exit as Exit>::SpotId>(value) {
                Ok(sp) => ctx.set_position_raw(sp),
                Err(e) => errs.push(format!("{}: {}", name, e)),
            },
            _ => {
                if !ignore_unrecognized {
                    errs.push(format!("{}: Unrecognized key {:?}", name, key));
                }
            }
        }
    }
}

macro_rules! assign_mode_or_append_err {
    ($mode:expr, $errs:expr, $val:expr) => {
        match $val {
            Ok(m) => $mode = Some(m),
            Err(e) => $errs.push(e),
        }
    };
}

fn build_test<'a, W, T>(
    yaml: &'a Yaml,
    initial: &T,
    name: &str,
) -> Result<Unittest<'a, T>, Vec<String>>
where
    T: Ctx<World = W>,
    W: World,
{
    let mut errs = Vec::new();
    let mut ctx = initial.clone();
    if let Some(tmap) = yaml.as_hash() {
        let mut test_name = None;
        let mut mode: Option<TestMode<T>> = None;
        apply_test_setup::<W, T>(&mut ctx, yaml, name, &mut errs, true);

        let obtainable = |can, value, name| match item_from_yaml::<T>(value) {
            Ok((item, ct)) => {
                if ct == 1 {
                    Ok(TestMode::Obtainable(can, item))
                } else {
                    Err(format!(
                        "{}: item count not accepted here: {:?}",
                        name, value,
                    ))
                }
            }
            Err(e) => Err(e),
        };

        for (key, value) in tmap {
            let tname = test_name.unwrap_or(name);
            match key.as_str() {
                Some("name") => {
                    if let Some(n) = value.as_str() {
                        test_name = Some(n);
                    } else {
                        errs.push(format!("{}: name must be string: {:?}", name, value));
                    }
                }
                Some("can_obtain") => {
                    assign_mode_or_append_err!(mode, errs, obtainable(true, value, tname))
                }
                Some("cannot_obtain") => {
                    assign_mode_or_append_err!(mode, errs, obtainable(false, value, tname))
                }

                _ => errs.push(format!("{}: key must be string: {:?}", name, key)),
            }
        }

        match (test_name, mode) {
            (Some(tn), Some(m)) => {
                return Ok(Unittest {
                    name: String::from(tn),
                    initial: ctx,
                    mode: m,
                });
            }
            (None, _) => errs.push(format!("{}: Please provide a name for this test", name)),
            (Some(tn), None) => errs.push(format!("{}: No test declared", tn)),
        }
    } else {
        errs.push(format!("{}: Expected key-value map", name));
    }
    Err(errs)
}

fn build_tests<'a, W, T>(
    yaml: &'a Yaml,
    initial: &T,
    name: &str,
) -> Result<Vec<Unittest<'a, T>>, Vec<String>>
where
    T: Ctx<World = W>,
    W: World,
{
    let mut errs = Vec::new();
    let mut unittests = Vec::new();
    if let Some(tests) = yaml.as_vec() {
        for (i, test) in tests.iter().enumerate() {
            match build_test(test, initial, &format!("{} test {}", name, i)) {
                Ok(unittest) => unittests.push(unittest),
                Err(e) => errs.extend(e),
            }
        }
    } else {
        errs.push(format!("{}: Value is not list: {:?}", name, yaml));
    }
    if errs.is_empty() {
        Ok(unittests)
    } else {
        Err(errs)
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
        match key.as_str() {
            Some("all") => apply_test_setup(&mut ctx, &value, "all", &mut errs, false),
            Some("tests") => match build_tests(value, &ctx, "tests") {
                Ok(u) => tests.extend(u),
                Err(e) => errs.extend(e),
            },
            Some(_) => errs.push(format!("Unrecognized top-level key: {:?}", key)),
            None => errs.push(format!("Top-level keys must be string: {:?}", key)),
        }
    }
}
