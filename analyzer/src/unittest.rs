#![allow(unused)]

use crate::context::*;
use crate::route::*;
use crate::world::*;
use crate::*;
use lazy_static::lazy_static;
use libtest_mimic::*;
use regex::Regex;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use yaml_rust::*;

pub enum TestMode<'a, T>
where
    T: Ctx,
{
    // We can probably pare down this list by changing bool to another enum:
    // immediate, not immediate, and eventually (in iteration limit)
    Obtainable(bool, T::ItemId),
    Reachable(bool, <<T::World as World>::Exit as Exit>::SpotId),
    Accessible(bool, <<T::World as World>::Location as Location>::LocId),
    Activatable(bool, <<T::World as World>::Action as Action>::ActionId),
    EventuallyGets(T::ItemId),
    EventuallyReaches(<<T::World as World>::Exit as Exit>::SpotId),
    EventuallyAccesses(<<T::World as World>::Location as Location>::LocId),
    EventuallyActivates(<<T::World as World>::Action as Action>::ActionId),
    Route(Vec<(HistoryAlias<T>, &'a str)>),
    RequiresToObtain(T, T::ItemId),
    RequiresToReach(T, <<T::World as World>::Exit as Exit>::SpotId),
    RequiresToAccess(T, <<T::World as World>::Location as Location>::LocId),
    RequiresToActivate(T, <<T::World as World>::Action as Action>::ActionId),
    EventuallyRequiresToObtain(T, T::ItemId, u32),
    EventuallyRequiresToReach(T, <<T::World as World>::Exit as Exit>::SpotId, u32),
    EventuallyRequiresToAccess(T, <<T::World as World>::Location as Location>::LocId, u32),
    EventuallyRequiresToActivate(T, <<T::World as World>::Action as Action>::ActionId, u32),
}

impl<'a, T> fmt::Display for TestMode<'a, T>
where
    T: Ctx,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TestMode::Obtainable(b, item) => {
                write!(f, "{}Obtainable({})", if *b { "" } else { "Not" }, item)
            }
            TestMode::Reachable(b, spot) => {
                write!(f, "{}Reachable({})", if *b { "" } else { "Not" }, spot)
            }
            TestMode::Accessible(b, loc) => {
                write!(f, "{}Accessible({})", if *b { "" } else { "Not" }, loc)
            }
            TestMode::Activatable(b, act) => {
                write!(f, "{}Activatable({})", if *b { "" } else { "Not" }, act)
            }
            TestMode::EventuallyGets(item) => write!(f, "EventuallyGets({})", item),
            TestMode::EventuallyReaches(spot) => write!(f, "EventuallyReaches({})", spot),
            TestMode::EventuallyAccesses(loc_id) => write!(f, "EventuallyAccesses({})", loc_id),
            TestMode::EventuallyActivates(act) => write!(f, "EventuallyActivates({})", act),
            TestMode::Route(v) => write!(
                f,
                "Path({})",
                v.iter()
                    .map(|(h, _)| format!("{:?}", h))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            TestMode::RequiresToObtain(_, item) => write!(f, "RequiresToObtain(<ctx>, {})", item),
            TestMode::RequiresToReach(_, spot) => write!(f, "RequiresToReach(<ctx>, {})", spot),
            TestMode::RequiresToAccess(_, loc_id) => {
                write!(f, "RequiresToAccess(<ctx>, {})", loc_id)
            }
            TestMode::RequiresToActivate(_, act) => write!(f, "RequiresToActivate(<ctx>, {})", act),

            TestMode::EventuallyRequiresToObtain(_, item, limit) => {
                write!(f, "EventuallyRequiresToObtain(<ctx>, {}, {})", item, limit)
            }
            TestMode::EventuallyRequiresToReach(_, spot, limit) => {
                write!(f, "EventuallyRequiresToReach(<ctx>, {}, {})", spot, limit)
            }
            TestMode::EventuallyRequiresToAccess(_, loc_id, limit) => {
                write!(
                    f,
                    "EventuallyRequiresToAccess(<ctx>, {}, {})",
                    loc_id, limit
                )
            }
            TestMode::EventuallyRequiresToActivate(_, act, limit) => {
                write!(f, "EventuallyRequiresToActivate(<ctx>, {}, {})", act, limit)
            }
        }
    }
}

pub struct Unittest<'a, T>
where
    T: Ctx,
{
    pub name: String,
    pub initial: T,
    pub mode: TestMode<'a, T>,
}

lazy_static! {
    static ref ITEM_RE: Regex = Regex::new(r"(\w+)\s*(?:\{(\d+)})?").unwrap();
}

fn item_from_yaml<T>(yaml: &Yaml) -> anyhow::Result<(T::ItemId, u32), String>
where
    T: Ctx,
{
    if let Some(s) = yaml.as_str() {
        if let Some(caps) = ITEM_RE.captures(s) {
            let item = T::ItemId::from_str(&caps[1])?;
            let ct = match caps.get(2) {
                Some(m) => u32::from_str(&m.as_str()).map_err(|e| format!("{:?}", e))?,
                None => 1,
            };
            Ok((item, ct))
        } else {
            Err(format!("Value did not parse: {:?}", yaml))
        }
    } else {
        Err(format!("Item value not a string: {:?}", yaml))
    }
}

fn item_only_from_yaml<T>(yaml: &Yaml) -> anyhow::Result<T::ItemId, String>
where
    T: Ctx,
{
    if let Some(s) = yaml.as_str() {
        if let Some(caps) = ITEM_RE.captures(s) {
            let item = T::ItemId::from_str(&caps[1])?;
            if caps.get(2).is_none() {
                Ok(item)
            } else {
                Err(format!("item count not accepted here: {:?}", yaml))
            }
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
        Err(format!("Obj value not a string: {:?}", yaml))
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
        let name = format!("{}.{}", name, key.as_str().unwrap_or("?"));
        match key.as_str() {
            Some("with") => {
                handle_with(value, ctx, &name, errs);
            }
            // these should be separate, but eh
            Some("context") | Some("settings") => {
                handle_context_values(ctx, value, &name, errs);
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
            Some(_) => {
                if !ignore_unrecognized {
                    errs.push(format!("{}: Unrecognized key {:?}", name, key));
                }
            }
            None => {
                errs.push(format!("{}: Key expected to be string: {:?}", name, key));
            }
        }
    }
}

macro_rules! assign_mode_or_append_err {
    ($mode:expr, $errs:expr, $val:expr) => {
        match $val {
            Ok(m) => {
                if let Some(m1) = &$mode {
                    $errs.push(format!("Multiple test defs: 1. {}  2. {}", m1, m));
                } else {
                    $mode = Some(m);
                }
            }
            Err(e) => $errs.push(e),
        }
    };

    ($mode:expr, $errs:expr, $val:expr, $name:expr) => {
        match $val {
            Ok(m) => {
                if let Some(m1) = &$mode {
                    $errs.push(format!(
                        "{}: Multiple test defs: 1. {}  2. {}",
                        $name, m1, m
                    ));
                } else {
                    $mode = Some(m);
                }
            }
            Err(e) => $errs.push(format!("{}: {}", $name, e)),
        }
    };
}

fn handle_requires_test<'a, W, T>(
    yaml: &'a Yaml,
    initial: &T,
    name: &str,
    eventually: bool,
) -> Result<TestMode<'a, T>, String>
where
    T: Ctx<World = W>,
    W: World,
    W::Location: Location<Context = T>,
{
    let mut rctx = initial.clone();
    let mut errs = Vec::new();
    apply_test_setup(&mut rctx, yaml, name, &mut errs, true);
    if let Some(map) = yaml.as_hash() {
        let mut mode: Option<TestMode<T>> = None;
        let mut ilimit = 1000;

        for (key, value) in map {
            match key.as_str() {
                Some("to_obtain" | "to_get") => {
                    assign_mode_or_append_err!(
                        mode,
                        errs,
                        item_only_from_yaml::<T>(value).map(|item| if eventually {
                            TestMode::EventuallyRequiresToObtain(rctx.clone(), item, ilimit)
                        } else {
                            TestMode::RequiresToObtain(rctx.clone(), item)
                        }),
                        name
                    )
                }
                Some("to_reach") => {
                    assign_mode_or_append_err!(
                        mode,
                        errs,
                        obj_from_yaml::<<W::Exit as Exit>::SpotId>(value).map(|sp| if eventually {
                            TestMode::EventuallyRequiresToReach(rctx.clone(), sp, ilimit)
                        } else {
                            TestMode::RequiresToReach(rctx.clone(), sp)
                        }),
                        name
                    )
                }
                Some("to_access") => {
                    assign_mode_or_append_err!(
                        mode,
                        errs,
                        obj_from_yaml::<<W::Location as Location>::LocId>(value).map(|loc_id| {
                            if eventually {
                                TestMode::EventuallyRequiresToAccess(rctx.clone(), loc_id, ilimit)
                            } else {
                                TestMode::RequiresToAccess(rctx.clone(), loc_id)
                            }
                        })
                    )
                }
                Some("to_activate") => {
                    assign_mode_or_append_err!(
                        mode,
                        errs,
                        obj_from_yaml::<<W::Action as Action>::ActionId>(value).map(|act| {
                            if eventually {
                                TestMode::EventuallyRequiresToActivate(rctx.clone(), act, ilimit)
                            } else {
                                TestMode::RequiresToActivate(rctx.clone(), act)
                            }
                        })
                    )
                }
                Some("iteration_limit") => {
                    if let Some(v) = value.as_i64() {
                        match u32::try_from(v) {
                            Ok(u) => ilimit = u,
                            Err(e) => errs.push(format!("{}.iteration_limit: {}", name, e)),
                        }
                    }
                }
                _ => {}
            }
        }
        match mode {
            Some(TestMode::EventuallyRequiresToObtain(c, item, _)) => {
                return Ok(TestMode::EventuallyRequiresToObtain(c, item, ilimit))
            }
            Some(TestMode::EventuallyRequiresToReach(c, spot, _)) => {
                return Ok(TestMode::EventuallyRequiresToReach(c, spot, ilimit))
            }
            Some(TestMode::EventuallyRequiresToAccess(c, loc_id, _)) => {
                return Ok(TestMode::EventuallyRequiresToAccess(c, loc_id, ilimit))
            }
            Some(TestMode::EventuallyRequiresToActivate(c, act, _)) => {
                return Ok(TestMode::EventuallyRequiresToActivate(c, act, ilimit))
            }
            Some(m) => return Ok(m),
            _ => errs.push(format!(
                "No test mode specified for {}requires",
                if eventually { "eventually_" } else { "" },
            )),
        }
    }
    Err(errs.join("\n"))
}

pub fn build_test<'a, W, T>(
    yaml: &'a Yaml,
    initial: &T,
    name: &str,
) -> Result<Unittest<'a, T>, Vec<String>>
where
    T: Ctx<World = W>,
    W: World,
    W::Location: Location<Context = T>,
{
    let mut errs = Vec::new();
    let mut ctx = initial.clone();
    if let Some(tmap) = yaml.as_hash() {
        let mut test_name = None;
        let mut mode: Option<TestMode<T>> = None;
        apply_test_setup::<W, T>(&mut ctx, yaml, name, &mut errs, true);

        let obtainable = |can, value| {
            item_only_from_yaml::<T>(value).map(|item| TestMode::Obtainable(can, item))
        };
        let reachable = |can, yaml| {
            obj_from_yaml::<<W::Exit as Exit>::SpotId>(yaml)
                .map(|sp| TestMode::<T>::Reachable(can, sp))
        };
        let accessible = |can, yaml| {
            obj_from_yaml::<<W::Location as Location>::LocId>(yaml)
                .map(|loc_id| TestMode::<T>::Accessible(can, loc_id))
        };
        let activatable = |can, yaml| {
            obj_from_yaml::<<W::Action as Action>::ActionId>(yaml)
                .map(|act| TestMode::<T>::Activatable(can, act))
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
                Some("can_obtain" | "can_get") => {
                    assign_mode_or_append_err!(mode, errs, obtainable(true, value), tname)
                }
                Some("cannot_obtain" | "cannot_get") => {
                    assign_mode_or_append_err!(mode, errs, obtainable(false, value), tname)
                }
                Some("can_reach") => {
                    assign_mode_or_append_err!(mode, errs, reachable(true, value), tname)
                }
                Some("cannot_reach") => {
                    assign_mode_or_append_err!(mode, errs, reachable(false, value), tname)
                }
                Some("can_access") => {
                    assign_mode_or_append_err!(mode, errs, accessible(true, value), tname)
                }
                Some("cannot_access") => {
                    assign_mode_or_append_err!(mode, errs, accessible(false, value), tname)
                }
                Some("can_activate") => {
                    assign_mode_or_append_err!(mode, errs, activatable(true, value), tname)
                }
                Some("cannot_activate") => {
                    assign_mode_or_append_err!(mode, errs, activatable(false, value), tname)
                }
                Some("eventually_gets" | "eventually_obtains") => {
                    assign_mode_or_append_err!(
                        mode,
                        errs,
                        item_only_from_yaml::<T>(value).map(|item| TestMode::EventuallyGets(item)),
                        tname
                    )
                }
                Some("eventually_reaches") => {
                    assign_mode_or_append_err!(
                        mode,
                        errs,
                        obj_from_yaml::<<W::Exit as Exit>::SpotId>(value)
                            .map(|sp| TestMode::EventuallyReaches(sp)),
                        tname
                    )
                }
                Some("eventually_accesses") => {
                    assign_mode_or_append_err!(
                        mode,
                        errs,
                        obj_from_yaml::<<W::Location as Location>::LocId>(value)
                            .map(|loc_id| TestMode::EventuallyAccesses(loc_id)),
                        tname
                    )
                }
                Some("eventually_activates") => {
                    assign_mode_or_append_err!(
                        mode,
                        errs,
                        obj_from_yaml::<<W::Action as Action>::ActionId>(value)
                            .map(|act| TestMode::EventuallyActivates(act)),
                        tname
                    )
                }
                Some("path") => assign_mode_or_append_err!(
                    mode,
                    errs,
                    match value {
                        Yaml::String(s) => histlines_from_string::<W, T, W::Location>(s),
                        Yaml::Array(v) => histlines_from_yaml_vec::<W, T, W::Location>(v),
                        _ => Err(String::from("Expected string or vec for path value")),
                    }
                    .map(|route| TestMode::Route(route)),
                    tname
                ),
                Some("requires") => assign_mode_or_append_err!(
                    mode,
                    errs,
                    handle_requires_test(value, &ctx, tname, false)
                ),
                Some("eventually_requires") => assign_mode_or_append_err!(
                    mode,
                    errs,
                    handle_requires_test(value, &ctx, tname, true)
                ),

                Some(_) => {}
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
    W::Location: Location<Context = T>,
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
    W::Location: Location<Context = T>,
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
            Some("name") => {}
            Some("all") => apply_test_setup(&mut ctx, &value, "all", &mut errs, false),
            Some("tests") => match build_tests(value, &ctx, "tests") {
                Ok(u) => tests.extend(u),
                Err(e) => errs.extend(e),
            },
            Some(_) => errs.push(format!("Unrecognized top-level key: {:?}", key)),
            None => errs.push(format!("Top-level keys must be string: {:?}", key)),
        }
    }

    println!("Collected {} tests and {} errors for {:?}:", tests.len(), errs.len(), filename);
    for t in tests {
        println!("{}: {}", t.name, t.mode);
    }
    if !errs.is_empty() {
        println!("\nErrors");
        for e in errs {
            println!("{}", e);
        }
    }
}
