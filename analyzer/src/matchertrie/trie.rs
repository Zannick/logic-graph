#![allow(unused)]

use super::matcher::MatcherDispatch;
use super::matcher::Observable;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::MutexGuard;

pub struct Node<MultiMatcherType> {
    matchers: Vec<MultiMatcherType>,
}

impl<MultiMatcherType> Default for Node<MultiMatcherType> {
    fn default() -> Self {
        Self {
            matchers: Vec::new(),
        }
    }
}

impl<MultiMatcherType, StructType, ValueType> Debug for Node<MultiMatcherType>
where
    MultiMatcherType: Debug + MatcherDispatch<Node = Self, Struct = StructType, Value = ValueType>,
    ValueType: Debug,
    StructType: Debug + Observable,
    <StructType as Observable>::PropertyValue: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "matchers:[ {} ]",
            self.matchers
                .iter()
                .map(|m| format!("{:?}", m))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

pub struct MatcherTrie<MultiMatcherType> {
    root: MultiMatcherType,
}

impl<MultiMatcherType> Default for MatcherTrie<MultiMatcherType>
where
    MultiMatcherType: Default,
{
    fn default() -> Self {
        Self {
            root: MultiMatcherType::default(),
        }
    }
}

impl<MultiMatcherType> Debug for MatcherTrie<MultiMatcherType>
where
    MultiMatcherType: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "MatcherTrie {{ {} }}",
            format!("{:?}", self.root)
        ))
    }
}

impl<MultiMatcherType, StructType, ValueType> MatcherTrie<MultiMatcherType>
where
    MultiMatcherType:
        MatcherDispatch<Node = Node<MultiMatcherType>, Struct = StructType, Value = ValueType>,
    StructType: Observable,
    ValueType: Clone,
{
    /// Performs a lookup for all states similar to this one.
    pub fn lookup(&self, similar: &StructType) -> Vec<ValueType> {
        let mut vec = Vec::new();
        let (node, val) = self.root.lookup(similar);
        if let Some(v) = val {
            vec.push(v.clone());
        }
        if let Some(mut node) = node {
            let mut node_queue = VecDeque::new();
            node_queue.push_back(node);
            'outer: while let Some(node) = node_queue.pop_front() {
                let locked_node = node.lock().unwrap();
                for matcher in locked_node.matchers.iter() {
                    let (inner, val) = matcher.lookup(similar);
                    if let Some(v) = val {
                        vec.push(v.clone());
                    }
                    if let Some(n) = inner {
                        node_queue.push_back(n);
                    }
                }
            }
        }
        vec
    }

    pub fn insert(
        &mut self,
        root_observation: StructType::PropertyValue,
        observations: Vec<StructType::PropertyValue>,
        value: ValueType,
    ) {
        if let Some((last, most)) = observations.split_last() {
            let mut current_node = self.root.insert(&root_observation).unwrap();

            let (last, most) = observations.split_last().unwrap();
            'observe: for obs in most.into_iter() {
                let mut locked_node = current_node.lock().unwrap();
                for matcher in locked_node.matchers.iter_mut() {
                    if let Some(next) = matcher.insert(obs) {
                        drop(locked_node);
                        current_node = next;
                        continue 'observe;
                    }
                }
                // We didn't match a matcher so we have to make a new one.
                let (next, matcher) = MultiMatcherType::new(&obs);
                locked_node.matchers.push(matcher);
                drop(locked_node);
                current_node = next;
            }
            let mut locked_node = current_node.lock().unwrap();
            for matcher in locked_node.matchers.iter_mut() {
                if let Some(_) = matcher.insert(last) {
                    matcher.set_value(last, value);
                    return;
                }
            }
            // We didn't match a matcher, so we have to make a new one.
            let (_, mut matcher) = MultiMatcherType::new(last);
            matcher.set_value(last, value);
            locked_node.matchers.push(matcher);
        } else {
            self.root.insert(&root_observation);
            self.root.set_value(&root_observation, value);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::matchertrie::matcher::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum Position {
        Start,
        Middle,
        End,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Ctx {
        pub pos: Position,
        pub flasks: i8,
        pub flag: u16,
    }

    static CTX_1: Ctx = Ctx {
        pos: Position::Start,
        flasks: 1,
        flag: 0xF,
    };
    static CTX_2: Ctx = Ctx {
        pos: Position::Start,
        flasks: 2,
        flag: 0xB,
    };

    static CTX_TEST_1: Ctx = Ctx {
        pos: Position::Start,
        flasks: 1,
        flag: 0x9,
    };

    // An enum with a list of properties and value internals. Bitflags have both a mask and result.
    #[derive(Clone, Copy, Debug)]
    enum PropertyValue {
        Pos(Position),
        Flasks(i8),
        Flag { mask: u16, result: u16 },
    }

    // An enum with type-specific matchers.
    // Each property could be represented here multiple times if there are different types of observations,
    // e.g. one for plain lookup, one for masked lookup, two for cmp (ge/lt or le/gt)...
    #[derive(Debug)]
    enum MatcherMulti {
        LookupPosition(LookupMatcher<Node<Self>, Ctx, Position>),
        LookupFlasks(LookupMatcher<Node<Self>, Ctx, i8>),
        MaskLookupFlag(LookupMatcher<Node<Self>, Ctx, u16>, u16),
    }

    impl Default for MatcherMulti {
        fn default() -> Self {
            Self::LookupPosition(LookupMatcher::new())
        }
    }

    impl Observable for Ctx {
        type PropertyValue = PropertyValue;
    }
    // That enum needs to have impls of the dispatch trait.
    impl MatcherDispatch for MatcherMulti {
        type Node = Node<Self>;
        type Struct = Ctx;
        type Value = Ctx;
        fn new(pv: &PropertyValue) -> (Arc<Mutex<Node<Self>>>, Self) {
            match pv {
                PropertyValue::Pos(p) => {
                    let mut m = LookupMatcher::new();
                    let node = m.insert(*p);
                    (node, Self::LookupPosition(m))
                }
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

        fn lookup(&self, val: &Ctx) -> (Option<Arc<Mutex<Node<Self>>>>, Option<Ctx>) {
            match self {
                Self::LookupPosition(m) => m.lookup(val.pos),
                Self::LookupFlasks(m) => m.lookup(val.flasks),
                Self::MaskLookupFlag(m, mask) => m.lookup(val.flag & mask),
            }
        }

        fn insert(&mut self, pv: &PropertyValue) -> Option<Arc<Mutex<Node<Self>>>> {
            match (self, pv) {
                (Self::LookupPosition(m), PropertyValue::Pos(p)) => Some(m.insert(*p)),
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
                (Self::LookupPosition(m), PropertyValue::Pos(p)) => m.set_value(*p, value),
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

    fn make_trie() -> MatcherTrie<MatcherMulti> {
        let mut trie = MatcherTrie::default();
        let observations = vec![
            PropertyValue::Flag {
                mask: 0x9,
                result: 0x9,
            },
            PropertyValue::Flasks(1),
        ];
        trie.insert(
            PropertyValue::Pos(Position::Start),
            observations,
            CTX_1.clone(),
        );

        let observations = vec![
            PropertyValue::Flag {
                mask: 0x7,
                result: 0x3,
            },
            PropertyValue::Flasks(2),
        ];
        trie.insert(
            PropertyValue::Pos(Position::Start),
            observations,
            CTX_2.clone(),
        );
        trie
    }

    #[test]
    fn node_lookup1() {
        let trie = make_trie();

        if let MatcherMulti::LookupPosition(m) = &trie.root {
            assert!(m.contains_key(&Position::Start));
        } else {
            panic!("Root is wrong type: {:?}", trie);
        }
        let (node, val) = trie.root.lookup(&CTX_TEST_1);
        assert_eq!(None, val);
        let node = node.expect("Root has Start but not next");
        let lock1 = node.lock().unwrap();
        assert_eq!(2, lock1.matchers.len());
        if let MatcherMulti::MaskLookupFlag(m, mask) = &lock1.matchers[0] {
            assert!(m.contains_key(&0x9), "{:?}", m);
            assert_eq!(0x9, mask & CTX_TEST_1.flag);
        } else {
            panic!("First matcher is wrong type");
        }

        let (node2, val) = lock1.matchers[0].lookup(&CTX_TEST_1);
        assert_eq!(None, val);
        let node2 = node2.expect("Node 2 has flag but no node");
        drop(lock1);
        let lock2 = node2.lock().unwrap();
        assert_eq!(1, lock2.matchers.len());
        if let MatcherMulti::LookupFlasks(m) = &lock2.matchers[0] {
            assert!(m.contains_key(&1), "{:?}", m);
            assert_eq!(1, CTX_TEST_1.flasks);
        } else {
            panic!("First matcher on node 2 is wrong type");
        }
        let (node3, val) = lock2.matchers[0].lookup(&CTX_TEST_1);

        assert_eq!(Some(CTX_1.clone()), val);
    }

    #[test]
    fn node_lookup2() {
        let trie = make_trie();

        if let MatcherMulti::LookupPosition(m) = &trie.root {
            assert!(m.contains_key(&Position::Start));
        } else {
            panic!("Root is wrong type: {:?}", trie);
        }
        let (node, val) = trie.root.lookup(&CTX_2);
        assert_eq!(None, val);
        let node = node.expect("Root has Start but not next");
        let lock1 = node.lock().unwrap();
        assert_eq!(2, lock1.matchers.len());
        if let MatcherMulti::MaskLookupFlag(m, mask) = &lock1.matchers[0] {
            assert!(m.contains_key(&0x9), "{:?}", m);
            assert_eq!(0x9, mask & CTX_2.flag);
        } else {
            panic!("First matcher is wrong type");
        }

        let (node2, val) = lock1.matchers[0].lookup(&CTX_2);
        assert_eq!(None, val);
        let node2 = node2.expect("Node 2 has flag but no node");
        drop(lock1);
        let lock2 = node2.lock().unwrap();
        assert_eq!(1, lock2.matchers.len());
        if let MatcherMulti::LookupFlasks(m) = &lock2.matchers[0] {
            assert!(!m.contains_key(&2), "{:?}", m);
            assert_eq!(2, CTX_2.flasks);
        } else {
            panic!("First matcher on node 2 is wrong type");
        }

        drop(lock2);
        let lock1 = node.lock().unwrap();
        if let MatcherMulti::MaskLookupFlag(m, mask) = &lock1.matchers[1] {
            assert!(m.contains_key(&0x3), "{:?}", m);
            assert_eq!(0x3, mask & CTX_2.flag);
        } else {
            panic!("First matcher is wrong type");
        }

        let (node3, val) = lock1.matchers[1].lookup(&CTX_2);
        assert_eq!(None, val);
        let node3 = node3.expect("Node 2 has flag but no node");
        drop(lock1);
        let lock3 = node3.lock().unwrap();
        assert_eq!(1, lock3.matchers.len());
        if let MatcherMulti::LookupFlasks(m) = &lock3.matchers[0] {
            assert!(m.contains_key(&2), "{:?}", m);
            assert_eq!(2, CTX_2.flasks);
        } else {
            panic!("First matcher on node 2 is wrong type");
        }
        let (node4, val) = lock3.matchers[0].lookup(&CTX_2);

        assert_eq!(Some(CTX_2.clone()), val);
    }

    #[test]
    fn retrieve() {
        let trie = make_trie();

        let t2 = Ctx {
            pos: Position::Middle,
            flasks: 0,
            flag: 0,
        };

        assert_eq!(
            vec![CTX_1.clone()],
            trie.lookup(&CTX_TEST_1),
            "trie: {:?}",
            trie
        );
        assert_eq!(vec![CTX_2], trie.lookup(&CTX_2), "trie: {:?}", trie);
        assert_eq!(0, trie.lookup(&t2).len());
    }

    #[test]
    fn multiple() {
        let mut trie = make_trie();
        let observations = vec![PropertyValue::Flag {
            mask: 0x9,
            result: 0x9,
        }];
        let c3 = Ctx {
            pos: Position::Start,
            flasks: 1,
            flag: 0x19,
        };
        trie.insert(
            PropertyValue::Pos(Position::Start),
            observations,
            c3.clone(),
        );
        let results = trie.lookup(&c3);
        assert_eq!(2, results.len());
        assert_ne!(results[0], results[1]);
    }
}
