#![allow(unused)]

use crate::context::*;
use crate::world::*;
use indextree::Arena;
use indextree::Node;
use indextree::NodeId;
use std::collections::HashMap;
use std::hash::Hash;

struct HistoryNode<T>
where
    T: Ctx,
{
    pub ctx: ContextWrapper<T>,
    pub entry: History<T>,
    pub children: HashMap<History<T>, NodeId>,
}

pub struct HistoryTree<T>
where
    T: Ctx,
{
    arena: Arena<HistoryNode<T>>,
    trees: HashMap<History<T>, NodeId>,
}

impl<T> HistoryTree<T>
where
    T: Ctx,
{
    pub fn new() -> HistoryTree<T> {
        HistoryTree {
            arena: Arena::new(),
            trees: HashMap::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.arena.count()
    }

    pub fn is_empty(&self) -> bool {
        self.arena.is_empty()
    }

    pub fn get(&self, id: NodeId) -> &ContextWrapper<T> {
        &self.arena.get(id).unwrap().get().ctx
    }

    pub fn new_tree(&mut self, step: History<T>, ctx: ContextWrapper<T>) -> Option<NodeId> {
        if self.trees.contains_key(&step) {
            None
        } else {
            let node = HistoryNode {
                ctx,
                entry: step.clone(),
                children: HashMap::new(),
            };
            let id = self.arena.new_node(node);
            self.trees.insert(step, id);
            Some(id)
        }
    }

    pub fn insert(
        &mut self,
        parent: NodeId,
        step: History<T>,
        ctx: ContextWrapper<T>,
    ) -> Option<NodeId> {
        let parent_node = self.arena.get(parent).unwrap().get();
        if parent_node.children.contains_key(&step) {
            None
        } else {
            let id = self.arena.new_node(HistoryNode {
                ctx,
                entry: step.clone(),
                children: HashMap::new(),
            });
            let parent_node = self.arena.get_mut(parent).unwrap().get_mut();
            parent_node.children.insert(step, id);
            parent.append(id, &mut self.arena);
            Some(id)
        }
    }

    pub fn get_history(&self, id: NodeId) -> Vec<&History<T>> {
        let mut vec: Vec<&History<T>> = id
            .ancestors(&self.arena)
            .map(|n| &self.arena.get(n).unwrap().get().entry)
            .collect();
        vec.reverse();
        vec
    }

    pub fn get_history_str(&self, id: NodeId) -> String {
        let mut vec: Vec<String> = id
            .ancestors(&self.arena)
            .map(|n| self.arena.get(n).unwrap().get().entry.to_string())
            .collect();
        vec.reverse();
        vec.join("\n")
    }
}
