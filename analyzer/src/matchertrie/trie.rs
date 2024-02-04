#![allow(unused)]

use crate::matchertrie::matcher;
use std::marker::PhantomData;
use std::sync::MutexGuard;

pub struct Node<MultiMatcherType, ValueType, PropValueType> {
    matchers: Vec<MultiMatcherType>,
    phantom: PhantomData<(ValueType, PropValueType)>,
}

impl<MultiMatcherType, ValueType, PropValueType> Default
    for Node<MultiMatcherType, ValueType, PropValueType>
{
    fn default() -> Self {
        Self {
            matchers: Vec::new(),
            phantom: PhantomData::default(),
        }
    }
}

pub struct MatcherTrie<MultiMatcherType, ValueType, PropValueType> {
    root: MultiMatcherType,
    phantom: PhantomData<(ValueType, PropValueType)>,
}

impl<MultiMatcherType, ValueType, PropValueType>
    MatcherTrie<MultiMatcherType, ValueType, PropValueType>
where
    MultiMatcherType: matcher::MatcherDispatch<
        Node<MultiMatcherType, ValueType, PropValueType>,
        ValueType,
        PropValueType,
    >,
{
    pub fn insert(
        &mut self,
        root_observation: PropValueType,
        observations: Vec<PropValueType>,
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
