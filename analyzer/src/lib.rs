extern crate bucket_queue;
extern crate clap;
extern crate enum_map;
extern crate libtest_mimic;
extern crate lru;
extern crate priority_queue;
extern crate rayon;
extern crate regex;
extern crate rmp_serde;
extern crate rustc_hash;
extern crate serde;
extern crate similar;
extern crate sort_by_derive;
extern crate yaml_rust;

mod a_star;
pub mod access;
pub mod bucket;
pub mod cli;
pub mod condense;
pub mod context;
pub mod db;
pub mod direct;
pub mod estimates;
pub mod greedy;
pub mod heap;
pub mod matchertrie;
pub mod minimize;
pub mod observer;
pub mod priority;
#[cfg(not(target_env = "msvc"))]
pub mod prof;
pub mod route;
pub mod scoring;
pub mod search;
pub mod settings;
pub mod solutions;
pub mod steiner;
pub mod storage;
mod timing;
pub mod world;

#[cfg(feature = "mysql")]
pub mod models;
#[cfg(feature = "mysql")]
pub mod schema;

// test-only
pub mod testlib;
pub mod unittest;

pub type CommonHasher = rustc_hash::FxBuildHasher;
pub fn new_hashmap<T, U>() -> std::collections::HashMap<T, U, CommonHasher> {
    rustc_hash::FxHashMap::default()
}
pub(crate) fn new_hashset<T>() -> std::collections::HashSet<T, CommonHasher> {
    rustc_hash::FxHashSet::default()
}
pub(crate) fn new_hashset_with<T>(el: T) -> std::collections::HashSet<T, CommonHasher>
where
    T: Eq + std::hash::Hash,
{
    let mut hs = new_hashset();
    hs.insert(el);
    hs
}
