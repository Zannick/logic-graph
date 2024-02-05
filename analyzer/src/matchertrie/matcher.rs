#![allow(unused)]

use crate::{new_hashmap, CommonHasher};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

/// Trait that marks the associated property-and-value type for observations.
pub trait MatcherStruct {
    type PropertyValue;
}

/// This is a trait to be implemented on enums with individual matcher types
pub trait MatcherDispatch {
    type Node;
    type Struct: MatcherStruct;
    type Value;
    /// Creates a new Matcher for the given Prop and Value.
    fn new(pv: &<Self::Struct as MatcherStruct>::PropertyValue) -> (Arc<Mutex<Self::Node>>, Self);

    /// The individual matcher will retrieve a property of the struct provided, and evaluate the value of that property.
    fn lookup(&self, val: &Self::Struct) -> (Option<Arc<Mutex<Self::Node>>>, Option<Self::Value>);

    fn insert(&mut self, pv: &<Self::Struct as MatcherStruct>::PropertyValue) -> Option<Arc<Mutex<Self::Node>>>;
    fn set_value(&mut self, pv: &<Self::Struct as MatcherStruct>::PropertyValue, value: Self::Value);
}

pub trait Matcher<NodeType, ValueType, IntType>
where
    IntType: Copy + Eq + Hash,
    ValueType: Clone,
{
    /// Performs a lookup of val against this matcher, and if there is a matching edge,
    /// returns a pointer to the next node if one exists and a reference to the value
    /// stored (if the path terminates). Both may exist, but usually if both do not exist,
    /// val was not a match.
    fn lookup(&self, val: IntType) -> (Option<Arc<Mutex<NodeType>>>, Option<ValueType>);

    /// Inserts matchers
    fn insert(&mut self, obs: IntType) -> Arc<Mutex<NodeType>>;
    fn set_value(&mut self, obs: IntType, value: ValueType);
}

pub struct LookupMatcher<NodeType, ValueType, IntType>
where
    IntType: Copy + Eq + Hash,
{
    map: HashMap<IntType, (Option<Arc<Mutex<NodeType>>>, Option<ValueType>), CommonHasher>,
}

impl<NodeType, ValueType, IntType> Debug for LookupMatcher<NodeType, ValueType, IntType>
where
    NodeType: Debug,
    IntType: Debug + Copy + Eq + Hash,
    ValueType: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "map: {{{}}}",
            self.map
                .iter()
                .map(|(k, (n, v))| format!(
                    "{:?} => ({}, {:?})",
                    k,
                    n.as_ref().map_or(String::from("No node"), |mutex| format!(
                        "{:?}",
                        mutex.lock().unwrap().deref()
                    )),
                    v
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

impl<NodeType, ValueType, IntType> Matcher<NodeType, ValueType, IntType>
    for LookupMatcher<NodeType, ValueType, IntType>
where
    NodeType: Default,
    IntType: Copy + Eq + Hash,
    ValueType: Clone,
{
    fn lookup(&self, val: IntType) -> (Option<Arc<Mutex<NodeType>>>, Option<ValueType>) {
        self.map
            .get(&val)
            .map_or((None, None), |(node, val)| (node.clone(), val.clone()))
    }

    fn insert(&mut self, obs: IntType) -> Arc<Mutex<NodeType>> {
        match self.map.get_mut(&obs) {
            Some((node, _)) => {
                if let Some(n) = node {
                    n.clone()
                } else {
                    let n: Arc<Mutex<NodeType>> = Arc::default();
                    *node = Some(n.clone());
                    n
                }
            }
            None => {
                let n: Arc<Mutex<NodeType>> = Arc::default();
                self.map.insert(obs, (Some(n.clone()), None));
                n
            }
        }
    }
    fn set_value(&mut self, obs: IntType, value: ValueType) {
        match self.map.get_mut(&obs) {
            Some((_, val)) => {
                if let None = val {
                    *val = Some(value);
                } else {
                    log::debug!("Replacing a value in matcher trie");
                    *val = Some(value);
                }
            }
            None => {
                self.map.insert(obs, (Some(Arc::default()), Some(value)));
            }
        }
    }
}

impl<NodeType, ValueType, IntType> LookupMatcher<NodeType, ValueType, IntType>
where
    NodeType: Default,
    IntType: Copy + Eq + Hash,
{
    pub fn new() -> Self {
        Self { map: new_hashmap() }
    }

    pub fn contains_key(&self, key: &IntType) -> bool {
        self.map.contains_key(key)
    }
}
