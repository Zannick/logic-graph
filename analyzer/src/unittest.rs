#![allow(unused)]

use crate::context::*;
use crate::estimates::ContextScorer;
use crate::route::*;
use crate::steiner::*;
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
use std::sync::Arc;
use yaml_rust::*;

pub enum TestMode<T>
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
    Route(Vec<(HistoryAlias<T>, String)>),
    RequiresToObtain(T, T::ItemId),
    RequiresToReach(T, <<T::World as World>::Exit as Exit>::SpotId),
    RequiresToAccess(T, <<T::World as World>::Location as Location>::LocId),
    RequiresToActivate(T, <<T::World as World>::Action as Action>::ActionId),
    // TODO: These require expectations provided in the unittest.
    EventuallyRequiresToObtain(T::ItemId, u32),
    EventuallyRequiresToReach(<<T::World as World>::Exit as Exit>::SpotId, u32),
    EventuallyRequiresToAccess(<<T::World as World>::Location as Location>::LocId, u32),
    EventuallyRequiresToActivate(<<T::World as World>::Action as Action>::ActionId, u32),
}

const DEFAULT_ITERATION_LIMIT: u32 = 100;

impl<T> fmt::Display for TestMode<T>
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

            TestMode::EventuallyRequiresToObtain(item, limit) => {
                write!(f, "EventuallyRequiresToObtain({}, {})", item, limit)
            }
            TestMode::EventuallyRequiresToReach(spot, limit) => {
                write!(f, "EventuallyRequiresToReach({}, {})", spot, limit)
            }
            TestMode::EventuallyRequiresToAccess(loc_id, limit) => {
                write!(f, "EventuallyRequiresToAccess({}, {})", loc_id, limit)
            }
            TestMode::EventuallyRequiresToActivate(act, limit) => {
                write!(f, "EventuallyRequiresToActivate({}, {})", act, limit)
            }
        }
    }
}

pub struct Unittest<T>
where
    T: Ctx,
{
    pub name: String,
    pub initial: T,
    pub mode: TestMode<T>,
    pub expects: Vec<T::Expectation>,
}

lazy_static! {
    static ref ITEM_RE: Regex = Regex::new(r"(\w+)\s*(?:\{(\d+)})?").unwrap();
    static ref DEFAULT_NAME_RE: Regex =
        Regex::new(r"Hash\(|String\(|Boolean\(|Integer\(|\W+").unwrap();
}

fn default_name(yaml: &Yaml) -> String {
    match yaml {
        Yaml::Real(r) => r.clone(),
        Yaml::Integer(i) => format!("{}", i),
        Yaml::String(s) => s.clone(),
        Yaml::Boolean(b) => format!("{}", b),
        Yaml::Array(a) => a.iter().map(default_name).collect::<Vec<_>>().join(","),
        Yaml::Hash(h) => h
            .iter()
            .map(|(k, v)| format!("{}:{}", default_name(k), default_name(v)))
            .collect::<Vec<_>>()
            .join(","),
        _ => String::from("yamlerror"),
    }
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

macro_rules! assign_mode_expects_or_append_err {
    ($mode:expr, $expects:expr, $errs:expr, $val:expr) => {
        match $val {
            Ok((m, xps)) => {
                if let Some(m1) = &$mode {
                    $errs.push(format!("Multiple test defs: 1. {}  2. {}", m1, m));
                } else {
                    $mode = Some(m);
                    $expects = xps;
                }
            }
            Err(e) => $errs.push(e),
        }
    };
}

fn handle_requires_test<'a, W, T>(
    yaml: &'a Yaml,
    initial: &T,
    name: &str,
    eventually: bool,
) -> Result<(TestMode<T>, Vec<T::Expectation>), String>
where
    T: Ctx<World = W>,
    W: World,
    W::Location: Location<Context = T>,
{
    let mut rctx = initial.clone();
    let mut errs = Vec::new();
    let mut expects = Vec::new();
    apply_test_setup(&mut rctx, yaml, name, &mut errs, true);
    if let Some(map) = yaml.as_hash() {
        let mut mode: Option<TestMode<T>> = None;
        let mut ilimit = DEFAULT_ITERATION_LIMIT;

        for (key, value) in map {
            match key.as_str() {
                Some("to_obtain" | "to_get") => {
                    assign_mode_or_append_err!(
                        mode,
                        errs,
                        item_only_from_yaml::<T>(value).map(|item| if eventually {
                            TestMode::EventuallyRequiresToObtain(item, ilimit)
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
                            TestMode::EventuallyRequiresToReach(sp, ilimit)
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
                                TestMode::EventuallyRequiresToAccess(loc_id, ilimit)
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
                                TestMode::EventuallyRequiresToActivate(act, ilimit)
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
                Some("expect") => match handle_expectations::<T>(value, name) {
                    Ok(v) => expects = v,
                    Err(e) => errs.extend(e),
                },
                _ => {}
            }
        }
        match mode {
            Some(TestMode::EventuallyRequiresToObtain(item, _)) => {
                return Ok((TestMode::EventuallyRequiresToObtain(item, ilimit), expects))
            }
            Some(TestMode::EventuallyRequiresToReach(spot, _)) => {
                return Ok((TestMode::EventuallyRequiresToReach(spot, ilimit), expects))
            }
            Some(TestMode::EventuallyRequiresToAccess(loc_id, _)) => {
                return Ok((
                    TestMode::EventuallyRequiresToAccess(loc_id, ilimit),
                    expects,
                ))
            }
            Some(TestMode::EventuallyRequiresToActivate(act, _)) => {
                return Ok((TestMode::EventuallyRequiresToActivate(act, ilimit), expects))
            }
            Some(m) => return Ok((m, expects)),
            _ => errs.push(format!(
                "No test mode specified for {}requires",
                if eventually { "eventually_" } else { "" },
            )),
        }
    }
    Err(errs.join("\n"))
}

fn handle_expectations<T>(yaml: &Yaml, name: &str) -> Result<Vec<T::Expectation>, Vec<String>>
where
    T: Ctx,
{
    let mut errs = Vec::new();
    let mut vec = Vec::new();
    if let Some(map) = yaml.as_hash() {
        for (key, value) in map {
            if let Some(s) = key.as_str() {
                match T::parse_expect_context(s, value) {
                    Ok(exp) => vec.push(exp),
                    Err(e) => errs.push(format!("{}: {}", name, e)),
                }
            } else {
                errs.push(format!("{}: key must be string: {:?}", name, key))
            }
        }
    } else {
        errs.push(format!("{}: Expected key-value map", name));
    }
    if errs.is_empty() {
        Ok(vec)
    } else {
        Err(errs)
    }
}

pub fn build_test<'a, W, T>(
    yaml: &'a Yaml,
    initial: &T,
    name: &str,
) -> Result<Unittest<T>, Vec<String>>
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
        let mut expects = Vec::new();
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
                Some("path") | Some("route") => assign_mode_or_append_err!(
                    mode,
                    errs,
                    match value {
                        Yaml::String(s) => histlines_from_string::<W, T, W::Location>(s),
                        Yaml::Array(v) => histlines_from_yaml_vec::<W, T, W::Location>(v),
                        _ => Err(String::from("Expected string or vec for path value")),
                    }
                    .map(|route| TestMode::Route(
                        route
                            .into_iter()
                            .map(|(h, s)| (h, format!("{}", s)))
                            .collect()
                    )),
                    tname
                ),
                Some("requires") => assign_mode_expects_or_append_err!(
                    mode,
                    expects,
                    errs,
                    handle_requires_test(value, &ctx, tname, false)
                ),
                Some("eventually_requires") => assign_mode_expects_or_append_err!(
                    mode,
                    expects,
                    errs,
                    handle_requires_test(value, &ctx, tname, true)
                ),
                Some("expect") => match handle_expectations::<T>(value, tname) {
                    Ok(v) => expects = v,
                    Err(e) => errs.extend(e),
                },

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
                    expects,
                });
            }
            (None, Some(m)) => {
                return Ok(Unittest {
                    name: default_name(yaml),
                    initial: ctx,
                    mode: m,
                    expects,
                });
            }
            (_, None) => {
                if errs.is_empty() {
                    errs.push(format!("{}: No test declared", test_name.unwrap_or(name)));
                }
            }
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
) -> Result<Vec<Unittest<T>>, Vec<String>>
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

pub fn run_test<W, T>(
    world: &W,
    mut initial: T,
    mode: TestMode<T>,
    expects: Vec<T::Expectation>,
    shortest_paths: &ShortestPaths<NodeId<W>, EdgeId<W>>,
) -> Result<(), String>
where
    T: Ctx<World = W>,
    W: World,
    W::Location: Location<Context = T>,
{
    let start = initial.position();
    match mode {
        TestMode::Obtainable(expected, item) => {
            if expected {
                expect_obtainable!(world, initial, start, item);
            } else {
                expect_not_obtainable!(world, initial, T, start, item);
            }
        }
        TestMode::Reachable(expected, spot) => {
            if expected {
                expect_any_route!(world, initial, start, spot);
            } else {
                expect_no_route!(world, initial, T, start, spot);
            }
        }
        TestMode::Accessible(expected, loc) => {
            if expected {
                expect_accessible!(world, initial, start, loc);
            } else {
                expect_inaccessible!(world, initial, T, start, loc);
            }
        }
        TestMode::Activatable(expected, action) => {
            if expected {
                expect_action_accessible!(world, initial, start, action);
            } else {
                expect_action_inaccessible!(world, initial, start, action);
            }
        }
        TestMode::EventuallyGets(item) => {
            expect_eventually_gets!(world, initial, start, item);
        }
        TestMode::EventuallyReaches(spot) => {
            expect_eventually_reaches!(world, initial, start, spot);
        }
        TestMode::EventuallyAccesses(loc) => {
            expect_eventually_accesses!(world, initial, start, loc);
        }
        TestMode::EventuallyActivates(action) => {
            expect_eventually_activates!(world, initial, start, action);
        }
        TestMode::Route(route) => {
            let mut ctx = ContextWrapper::new(initial);
            let mut output = Vec::new();

            for (i, (h, s)) in route.into_iter().enumerate() {
                let mut next =
                    step_from_route(ctx.clone(), i, h, world, &shortest_paths).map_err(|e| {
                        format!("Route summary:\n{}\nAt {}: {}", output.join("\n"), s, e)
                    })?;
                output.push(history_str::<T, _>(next.remove_history().0.into_iter()));
                output.push(next.get().diff(ctx.get()));
                ctx = next;
            }
            ctx.get().assert_expectations(&expects)?;
        }
        TestMode::RequiresToObtain(mut ctx2, item) => {
            expect_not_obtainable!(world, initial, T, start, item);
            expect_obtainable!(world, ctx2, start, item);
        }
        TestMode::RequiresToReach(mut ctx2, spot) => {
            expect_no_route!(world, initial, T, start, spot);
            expect_any_route!(world, ctx2, start, spot);
        }
        TestMode::RequiresToAccess(mut ctx2, loc) => {
            expect_inaccessible!(world, initial, T, start, loc);
            expect_accessible!(world, ctx2, start, loc);
        }
        TestMode::RequiresToActivate(mut ctx2, action) => {
            expect_action_inaccessible!(world, initial, start, action);
            expect_action_accessible!(world, ctx2, start, action);
        }
        TestMode::EventuallyRequiresToObtain(item, ilimit) => {
            expect_eventually_requires_to_obtain!(
                world,
                initial,
                T,
                start,
                item,
                |c: &T| T::assert_expectations(c, &expects),
                ilimit
            );
        }
        TestMode::EventuallyRequiresToReach(spot, ilimit) => {
            expect_eventually_requires_to_reach!(
                world,
                initial,
                T,
                start,
                spot,
                |c: &T| T::assert_expectations(c, &expects),
                ilimit
            )
        }
        TestMode::EventuallyRequiresToAccess(loc_id, ilimit) => {
            expect_eventually_requires_to_access!(
                world,
                initial,
                T,
                start,
                loc_id,
                |c: &T| T::assert_expectations(c, &expects),
                ilimit
            )
        }
        TestMode::EventuallyRequiresToActivate(act, ilimit) => {
            expect_eventually_requires_to_activate!(
                world,
                initial,
                T,
                start,
                act,
                |c: &T| T::assert_expectations(c, &expects),
                ilimit
            )
        }
    }
    Ok(())
}

pub fn parse_test_file<W, T>(
    world: Arc<Box<W>>,
    filename: &PathBuf,
    shortest_paths: Arc<Box<ShortestPaths<NodeId<W>, EdgeId<W>>>>,
) -> Vec<Trial>
where
    T: Ctx<World = W> + 'static,
    W: World + Send + 'static,
    W::Location: Location<Context = T>,
{
    let mut file = File::open(filename)
        .unwrap_or_else(|e| panic!("Couldn't open file \"{:?}\": {:?}", filename, e));
    let mut prefix = filename
        .file_stem()
        .and_then(|f| f.to_str())
        .unwrap_or_else(|| panic!("Filename error in \"{:?}\"", filename));
    let mut settings = String::new();
    file.read_to_string(&mut settings)
        .unwrap_or_else(|e| panic!("Couldn't read from file \"{:?}\": {:?}", filename, e));
    let yaml = YamlLoader::load_from_str(&settings).expect("YAML parse error");
    let mut errs = Vec::new();
    let mut ctx = T::default();
    let mut tests: Vec<Unittest<T>> = Vec::new();

    for (key, value) in yaml[0]
        .as_hash()
        .expect("YAML file should be a key-value map")
    {
        match key.as_str() {
            Some("name") => {
                if let Some(v) = value.as_str() {
                    prefix = v;
                }
            }
            Some("all") => apply_test_setup(
                &mut ctx,
                value,
                &format!("{}.all", prefix),
                &mut errs,
                false,
            ),
            Some("tests") => match build_tests(value, &ctx, &format!("{}.tests", prefix)) {
                Ok(u) => tests.extend(u),
                Err(e) => errs.extend(e),
            },
            Some(_) => errs.push(format!("Unrecognized top-level key: {:?}", key)),
            None => errs.push(format!("Top-level keys must be string: {:?}", key)),
        }
    }

    assert!(
        errs.is_empty(),
        "Errors while parsing {:?}:\n{}",
        filename,
        errs.join("\n")
    );
    tests
        .into_iter()
        .map(|t| {
            let wp = world.clone();
            let sp = shortest_paths.clone();
            Trial::test(format!("{}:{}", prefix, t.name), move || {
                Ok(run_test(&**wp, t.initial, t.mode, t.expects, &**sp)?)
            })
        })
        .collect()
}

pub fn parse_route_file<W, T>(
    world: Arc<Box<W>>,
    filename: &PathBuf,
    shortest_paths: Arc<Box<ShortestPaths<NodeId<W>, EdgeId<W>>>>,
) -> Trial
where
    T: Ctx<World = W> + 'static,
    W: World + Send + 'static,
    W::Location: Location<Context = T>,
{
    let mut file = File::open(filename)
        .unwrap_or_else(|e| panic!("Couldn't open file \"{:?}\": {:?}", filename, e));
    let mut prefix = filename
        .file_stem()
        .and_then(|f| f.to_str())
        .unwrap_or_else(|| panic!("Filename error in \"{:?}\"", filename));
    let mut route_str = String::new();
    file.read_to_string(&mut route_str)
        .unwrap_or_else(|e| panic!("Couldn't read from file \"{:?}\": {:?}", filename, e));

    let mode = histlines_from_string::<W, T, W::Location>(&route_str).map(|route| {
        TestMode::Route(
            route
                .into_iter()
                .map(|(h, s)| (h, format!("{}", s)))
                .collect(),
        )
    });
    Trial::test(format!("routes/{}", prefix), move || {
        Ok(run_test(
            &**world,
            T::default(),
            mode?,
            vec![],
            &**shortest_paths,
        )?)
    })
}

pub fn run_test_file<W, T>(
    world: Arc<Box<W>>,
    filename: &PathBuf,
    shortest_paths: Arc<Box<ShortestPaths<NodeId<W>, EdgeId<W>>>>,
) where
    T: Ctx<World = W> + 'static,
    W: World + Send + 'static,
    W::Location: Location<Context = T>,
{
    let tests = parse_test_file(world, filename, shortest_paths);
    let args = Arguments::from_args();
    run(&args, tests); //.exit_if_failed();
}

pub fn run_all_tests_in_dir<W, T>(dirname: &PathBuf, route_dir: Option<&PathBuf>)
where
    T: Ctx<World = W> + 'static,
    W: World + Send + 'static,
    W::Location: Location<Context = T>,
{
    let mut world = W::new();
    //world.condense_graph();
    let startctx = T::default();
    let shortest_paths = Arc::new(Box::new(ContextScorer::shortest_paths_tree_only(
        &*world, &startctx,
    )));

    let wp = Arc::new(world);
    let mut tests = Vec::new();

    for entry in std::fs::read_dir(dirname).unwrap() {
        let path = entry.unwrap().path();
        let ext = path.extension().and_then(|s| s.to_str());
        if matches!(ext, Some("yaml")) {
            tests.extend(parse_test_file::<W, T>(
                wp.clone(),
                &path,
                shortest_paths.clone(),
            ));
        }
    }

    if let Some(routedirname) = route_dir {
        if routedirname.exists() {
            for entry in std::fs::read_dir(routedirname).unwrap() {
                let path = entry.unwrap().path();
                let ext = path.extension().and_then(|s| s.to_str());
                if matches!(ext, Some("txt")) {
                    tests.push(parse_route_file::<W, T>(
                        wp.clone(),
                        &path,
                        shortest_paths.clone(),
                    ));
                }
            }
        }
    }

    let args = Arguments::from_args();
    run(&args, tests); //.exit_if_failed();
}
