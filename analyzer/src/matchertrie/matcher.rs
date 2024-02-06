//! Matcher traits and definitions for the Matcher Trie.
//!
//! When designing your PropertyObservation type and the MatcherDispatch type, some tips:
//!   1. Enums are probably best.
//!   2. It's easiest to build your `match` rules if each PropertyObservation element
//!      maps directly to exactly one MatcherDispatch element.
//!   3. PropertyObservations should contain two parts: the property and the observed value. E.g.:
//!        `Flask(i8)`, in which `Flask` is the property and `i8` is the (type of the) value.
//!        `FlaskGe{g: i8, res: bool}`, in which `Flask >= g` is the property, and `res` is the value.
//!      The property part is how you'll distinguish whether the matcher is the right one for
//!      `insert()` and `set_value()`, and the value part is the key into the correct matcher.

#![allow(unused)]

use crate::{new_hashmap, CommonHasher};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

/// Trait that marks the associated property-and-value type for observations.
pub trait Observable {
    type PropertyObservation;
}

/// This is a trait to be implemented on enums with individual matcher types
pub trait MatcherDispatch {
    type Node;
    type Struct: Observable;
    type Value;
    /// Creates a new Matcher for the given Prop and Value.
    fn new(
        pv: &<Self::Struct as Observable>::PropertyObservation,
    ) -> (Arc<Mutex<Self::Node>>, Self);

    /// The individual matcher will retrieve a property of the struct provided, and evaluate the value of that property.
    fn lookup(&self, val: &Self::Struct) -> (Option<Arc<Mutex<Self::Node>>>, Option<Self::Value>);

    fn insert(
        &mut self,
        pv: &<Self::Struct as Observable>::PropertyObservation,
    ) -> Option<Arc<Mutex<Self::Node>>>;
    fn set_value(
        &mut self,
        pv: &<Self::Struct as Observable>::PropertyObservation,
        value: Self::Value,
    );
}

pub trait Matcher<NodeType, KeyType, ValueType>
where
    KeyType: Copy + Eq + Hash,
    ValueType: Clone,
{
    /// Performs a lookup of val against this matcher, and if there is a matching edge,
    /// returns a pointer to the next node if one exists and a reference to the value
    /// stored (if the path terminates). Both may exist, but usually if both do not exist,
    /// val was not a match.
    fn lookup(&self, val: KeyType) -> (Option<Arc<Mutex<NodeType>>>, Option<ValueType>);

    /// Inserts matchers
    fn insert(&mut self, obs: KeyType) -> Arc<Mutex<NodeType>>;
    fn set_value(&mut self, obs: KeyType, value: ValueType);
}

#[derive(Default)]
pub struct LookupMatcher<NodeType, KeyType, ValueType>
where
    KeyType: Copy + Eq + Hash,
{
    map: HashMap<KeyType, (Option<Arc<Mutex<NodeType>>>, Option<ValueType>), CommonHasher>,
}

impl<NodeType, KeyType, ValueType> Debug for LookupMatcher<NodeType, KeyType, ValueType>
where
    NodeType: Debug,
    KeyType: Debug + Copy + Eq + Hash,
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

impl<NodeType, KeyType, ValueType> Matcher<NodeType, KeyType, ValueType>
    for LookupMatcher<NodeType, KeyType, ValueType>
where
    NodeType: Default,
    KeyType: Copy + Eq + Hash,
    ValueType: Clone,
{
    fn lookup(&self, val: KeyType) -> (Option<Arc<Mutex<NodeType>>>, Option<ValueType>) {
        self.map
            .get(&val)
            .map_or((None, None), |(node, val)| (node.clone(), val.clone()))
    }

    fn insert(&mut self, obs: KeyType) -> Arc<Mutex<NodeType>> {
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
    fn set_value(&mut self, obs: KeyType, value: ValueType) {
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

impl<NodeType, KeyType, ValueType> LookupMatcher<NodeType, KeyType, ValueType>
where
    NodeType: Default,
    KeyType: Copy + Eq + Hash,
{
    pub fn new() -> Self {
        Self { map: new_hashmap() }
    }

    pub fn contains_key(&self, key: &KeyType) -> bool {
        self.map.contains_key(key)
    }
}

impl<NodeType, KeyType, ValueType> LookupMatcher<NodeType, KeyType, ValueType>
where
    NodeType: Default,
    KeyType: Copy + Eq + Hash,
    ValueType: Clone,
{
    pub fn new_with(obs: KeyType) -> (Arc<Mutex<NodeType>>, Self) {
        let mut m = Self::new();
        (m.insert(obs), m)
    }
}

// Comparison matchers are inevitably binary: the test is whether they conform to the comparison.
// Thus, we defer the actual comparison to the MatcherDispatch, which shall pass the result to
// this matcher which is essentially a special case lookup with exactly two possible values.
#[derive(Default)]
pub struct BooleanMatcher<NodeType, ValueType> {
    true_node: Option<Arc<Mutex<NodeType>>>,
    true_value: Option<ValueType>,
    false_node: Option<Arc<Mutex<NodeType>>>,
    false_value: Option<ValueType>,
}

impl<NodeType, ValueType> Debug for BooleanMatcher<NodeType, ValueType>
where
    NodeType: Debug,
    ValueType: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{{ T => ({}, {:?}, F => ({}, {:?}) }}",
            self.true_node
                .as_ref()
                .map_or(String::from("No node"), |mutex| format!(
                    "{:?}",
                    mutex.lock().unwrap().deref()
                )),
            self.true_value,
            self.false_node
                .as_ref()
                .map_or(String::from("No node"), |mutex| format!(
                    "{:?}",
                    mutex.lock().unwrap().deref()
                )),
            self.false_value
        ))
    }
}

impl<NodeType, ValueType> Matcher<NodeType, bool, ValueType> for BooleanMatcher<NodeType, ValueType>
where
    NodeType: Default,
    ValueType: Clone,
{
    fn lookup(&self, obs: bool) -> (Option<Arc<Mutex<NodeType>>>, Option<ValueType>) {
        if obs {
            (self.true_node.clone(), self.true_value.clone())
        } else {
            (self.false_node.clone(), self.false_value.clone())
        }
    }

    fn insert(&mut self, obs: bool) -> Arc<Mutex<NodeType>> {
        let node = if obs {
            &mut self.true_node
        } else {
            &mut self.false_node
        };

        if let Some(n) = node {
            n.clone()
        } else {
            let n: Arc<Mutex<NodeType>> = Arc::default();
            *node = Some(n.clone());
            n
        }
    }

    fn set_value(&mut self, obs: bool, value: ValueType) {
        let val = if obs {
            &mut self.true_value
        } else {
            &mut self.false_value
        };

        if let None = val {
            *val = Some(value);
        } else {
            log::debug!("Replacing a value in matcher trie");
            *val = Some(value);
        }
    }
}

impl<NodeType, ValueType> BooleanMatcher<NodeType, ValueType> {
    pub fn new() -> Self {
        Self {
            true_node: None,
            true_value: None,
            false_node: None,
            false_value: None,
        }
    }
}

impl<NodeType, ValueType> BooleanMatcher<NodeType, ValueType>
where
    NodeType: Default,
    ValueType: Clone,
{
    pub fn new_with(obs: bool) -> (Arc<Mutex<NodeType>>, Self) {
        let mut m = Self::new();
        (m.insert(obs), m)
    }
}
