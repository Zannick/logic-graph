#![allow(unused)]

use crate::{new_hashmap, CommonHasher};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

/// This is a trait to be implemented on enums with individual matcher types
pub trait MatcherDispatch<NodeType, ValueType, PropValueType> {
    /// Creates a new Matcher for the given Prop and Value.
    fn new(pv: &PropValueType) -> (Arc<Mutex<NodeType>>, Self);

    /// The individual matcher will retrieve a property of the value provided, and evaluate the value of that property.
    fn lookup(&self, val: &ValueType) -> (Option<Arc<Mutex<NodeType>>>, Option<ValueType>);

    fn insert(&mut self, pv: &PropValueType) -> Option<Arc<Mutex<NodeType>>>;
    fn set_value(&mut self, pv: &PropValueType, value: ValueType);
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

// Draft implementations for the final handlers that will go in games/
// We already have the Context type.
#[derive(Clone)]
pub struct Ctx {
    pub flasks: i8,
    pub flag: u16,
}

// An enum with a list of properties and value internals. Bitflags have both a mask and result.
enum PropertyValue {
    Flasks(i8),
    Flag { mask: u16, result: u16 },
}

// This will probably become Node<MultiMatcherType> and then we just mark our MultiMatcher enum.
pub struct NodeT {
    matchers: Vec<MatcherMulti>,
}

impl Default for NodeT {
    fn default() -> Self {
        Self {
            matchers: Vec::new(),
        }
    }
}

// An enum with type-specific matchers.
// Each property could be represented here multiple times if there are different types of observations,
// e.g. one for plain lookup, one for masked lookup, two for cmp (ge/lt or le/gt)...
enum MatcherMulti {
    LookupFlasks(LookupMatcher<NodeT, Ctx, i8>),
    MaskLookupFlag(LookupMatcher<NodeT, Ctx, u16>, u16),
}

// That enum needs to have impls of the dispatch trait.
impl MatcherDispatch<NodeT, Ctx, PropertyValue> for MatcherMulti {
    fn new(pv: &PropertyValue) -> (Arc<Mutex<NodeT>>, Self) {
        match pv {
            PropertyValue::Flasks(f) => {
                let mut m = LookupMatcher::new();
                let node = m.insert(*f);
                (node, Self::LookupFlasks(m))
            }
            PropertyValue::Flag { mask, result } => {
                let mut m = LookupMatcher::new();
                let node = m.insert(*result);
                (node, Self::MaskLookupFlag(m, *mask))
            }
        }
    }

    fn lookup(&self, val: &Ctx) -> (Option<Arc<Mutex<NodeT>>>, Option<Ctx>) {
        match self {
            Self::LookupFlasks(m) => m.lookup(val.flasks),
            Self::MaskLookupFlag(m, mask) => m.lookup(val.flag & mask),
        }
    }

    fn insert(&mut self, pv: &PropertyValue) -> Option<Arc<Mutex<NodeT>>> {
        match (self, pv) {
            (Self::LookupFlasks(m), PropertyValue::Flasks(f)) => Some(m.insert(*f)),
            (Self::MaskLookupFlag(m, used_mask), PropertyValue::Flag { mask, result })
                if used_mask == mask =>
            {
                Some(m.insert(*result))
            }
            _ => None,
        }
    }

    fn set_value(&mut self, pv: &PropertyValue, value: Ctx) {
        match (self, pv) {
            (Self::LookupFlasks(m), PropertyValue::Flasks(f)) => m.set_value(*f, value),
            (Self::MaskLookupFlag(m, used_mask), PropertyValue::Flag { mask, result })
                if used_mask == mask =>
            {
                m.set_value(*result, value)
            }
            _ => (),
        }
    }
}
