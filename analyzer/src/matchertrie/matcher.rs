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

use crate::{new_hashmap, new_hashset, new_hashset_with, CommonHasher};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::empty;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

/// Trait that marks the associated property-and-value type for observations.
pub trait Observable {
    type PropertyObservation: Debug;

    fn matches(&self, obs: &Self::PropertyObservation) -> bool;
    fn matches_all(&self, observations: &[Self::PropertyObservation]) -> bool {
        observations.into_iter().all(|obs| self.matches(obs))
    }
}

/// This is a trait to be implemented on enums with individual matcher types
pub trait MatcherDispatch {
    type Node;
    type Struct: Observable;
    type Value;
    /// Creates a new Matcher for the given Prop and Value.
    fn new(
        obs: &<Self::Struct as Observable>::PropertyObservation,
    ) -> (Arc<Mutex<Self::Node>>, Self);

    /// Clears the data out of the matcher.
    fn clear(&mut self);

    /// The individual matcher will retrieve a property of the struct provided, and evaluate the value of that property.
    fn lookup(&self, val: &Self::Struct) -> (Option<Arc<Mutex<Self::Node>>>, Vec<Self::Value>);

    /// Creates a new node in the individual matcher.
    ///
    /// Implementations should only add if the observation is an exact match, and not merely within the same range.
    fn insert(
        &mut self,
        obs: &<Self::Struct as Observable>::PropertyObservation,
    ) -> Option<Arc<Mutex<Self::Node>>>;

    /// Adds a value to the existing node in this matcher, or creates a new node with the value.
    fn add_value(
        &mut self,
        obs: &<Self::Struct as Observable>::PropertyObservation,
        value: Self::Value,
    );
    /// Adds a value as above, but only adds if all values already existing here pass the test.
    fn add_value_if_all(
        &mut self,
        obs: &<Self::Struct as Observable>::PropertyObservation,
        value: Self::Value,
        test: impl FnMut(&Self::Value) -> bool,
    );

    fn nodes(&self) -> Vec<Arc<Mutex<Self::Node>>>;
    fn num_values(&self) -> usize;
}

pub trait Matcher<NodeType, KeyType, ValueType>
where
    KeyType: Copy + Eq + Hash,
    ValueType: Clone,
{
    /// Clears the data out of this matcher's nodes.
    fn clear(&mut self);

    /// Performs a lookup of obs against this matcher, and if there is a matching edge,
    /// returns a pointer to the next node if one exists and a reference to the value
    /// stored (if the path terminates). Both may exist, but usually if both do not exist,
    /// obs was not a match for this matcher.
    fn lookup(&self, obs: KeyType) -> (Option<Arc<Mutex<NodeType>>>, Vec<ValueType>);

    /// Inserts matchers
    fn insert(&mut self, obs: KeyType) -> Arc<Mutex<NodeType>>;
    fn add_value(&mut self, obs: KeyType, value: ValueType);
    fn add_value_if_all(
        &mut self,
        obs: KeyType,
        value: ValueType,
        test: impl FnMut(&ValueType) -> bool,
    );

    fn nodes(&self) -> Vec<Arc<Mutex<NodeType>>>;
    fn num_values(&self) -> usize;
}

#[derive(Default)]
pub struct LookupMatcher<NodeType, KeyType, ValueType>
where
    KeyType: Copy + Eq + Hash,
    ValueType: Eq + Hash,
{
    map: HashMap<
        KeyType,
        (
            Option<Arc<Mutex<NodeType>>>,
            HashSet<ValueType, CommonHasher>,
        ),
        CommonHasher,
    >,
}

impl<NodeType, KeyType, ValueType> Debug for LookupMatcher<NodeType, KeyType, ValueType>
where
    NodeType: Debug,
    KeyType: Debug + Copy + Eq + Hash,
    ValueType: Debug + Eq + Hash,
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
    ValueType: Clone + Eq + Hash,
{
    fn clear(&mut self) {
        self.map.clear();
    }

    fn lookup(&self, obs: KeyType) -> (Option<Arc<Mutex<NodeType>>>, Vec<ValueType>) {
        self.map
            .get(&obs)
            .map_or((None, Vec::new()), |(node, val)| {
                (node.clone(), val.iter().cloned().collect())
            })
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
                self.map.insert(obs, (Some(n.clone()), new_hashset()));
                n
            }
        }
    }
    fn add_value(&mut self, obs: KeyType, value: ValueType) {
        match self.map.get_mut(&obs) {
            Some((_, val)) => {
                val.insert(value);
            }
            None => {
                self.map
                    .insert(obs, (Some(Arc::default()), new_hashset_with(value)));
            }
        }
    }
    fn add_value_if_all(
        &mut self,
        obs: KeyType,
        value: ValueType,
        test: impl FnMut(&ValueType) -> bool,
    ) {
        match self.map.get_mut(&obs) {
            Some((_, val)) => {
                if val.iter().all(test) {
                    val.insert(value);
                }
            }
            None => {
                self.map
                    .insert(obs, (Some(Arc::default()), new_hashset_with(value)));
            }
        }
    }

    fn nodes(&self) -> Vec<Arc<Mutex<NodeType>>> {
        self.map.values().filter_map(|(n, _)| n.clone()).collect()
    }

    fn num_values(&self) -> usize {
        self.map.values().map(|(_, v)| v.len()).sum()
    }
}

impl<NodeType, KeyType, ValueType> LookupMatcher<NodeType, KeyType, ValueType>
where
    NodeType: Default,
    KeyType: Copy + Eq + Hash,
    ValueType: Eq + Hash,
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
    ValueType: Clone + Eq + Hash,
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
pub struct BooleanMatcher<NodeType, ValueType>
where
    ValueType: Eq + Hash,
{
    true_node: Option<Arc<Mutex<NodeType>>>,
    true_values: HashSet<ValueType, CommonHasher>,
    false_node: Option<Arc<Mutex<NodeType>>>,
    false_values: HashSet<ValueType, CommonHasher>,
}

impl<NodeType, ValueType> Debug for BooleanMatcher<NodeType, ValueType>
where
    NodeType: Debug,
    ValueType: Debug + Eq + Hash,
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
            self.true_values,
            self.false_node
                .as_ref()
                .map_or(String::from("No node"), |mutex| format!(
                    "{:?}",
                    mutex.lock().unwrap().deref()
                )),
            self.false_values
        ))
    }
}

impl<NodeType, ValueType> Matcher<NodeType, bool, ValueType> for BooleanMatcher<NodeType, ValueType>
where
    NodeType: Default,
    ValueType: Clone + Eq + Hash,
{
    fn clear(&mut self) {
        self.true_node = None;
        self.false_node = None;
        self.true_values.clear();
        self.false_values.clear();
    }

    fn lookup(&self, obs: bool) -> (Option<Arc<Mutex<NodeType>>>, Vec<ValueType>) {
        if obs {
            (
                self.true_node.clone(),
                self.true_values.iter().cloned().collect(),
            )
        } else {
            (
                self.false_node.clone(),
                self.false_values.iter().cloned().collect(),
            )
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

    fn add_value(&mut self, obs: bool, value: ValueType) {
        let val = if obs {
            &mut self.true_values
        } else {
            &mut self.false_values
        };

        val.insert(value);
    }
    fn add_value_if_all(
        &mut self,
        obs: bool,
        value: ValueType,
        test: impl FnMut(&ValueType) -> bool,
    ) {
        let val = if obs {
            &mut self.true_values
        } else {
            &mut self.false_values
        };

        if val.iter().all(test) {
            val.insert(value);
        }
    }

    fn nodes(&self) -> Vec<Arc<Mutex<NodeType>>> {
        let mut vec = Vec::new();
        if let Some(n) = &self.true_node {
            vec.push(n.clone());
        }
        if let Some(n) = &self.false_node {
            vec.push(n.clone());
        }
        vec
    }

    fn num_values(&self) -> usize {
        self.true_values.len() + self.false_values.len()
    }
}

impl<NodeType, ValueType> BooleanMatcher<NodeType, ValueType>
where
    ValueType: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            true_node: None,
            true_values: new_hashset(),
            false_node: None,
            false_values: new_hashset(),
        }
    }
}

impl<NodeType, ValueType> BooleanMatcher<NodeType, ValueType>
where
    NodeType: Default,
    ValueType: Clone + Eq + Hash,
{
    pub fn new_with(obs: bool) -> (Arc<Mutex<NodeType>>, Self) {
        let mut m = Self::new();
        (m.insert(obs), m)
    }
}
