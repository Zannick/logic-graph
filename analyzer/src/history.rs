use crate::context::*;
use crate::world::*;
use indextree::Arena;
use indextree::NodeId;
use std::collections::HashMap;

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

    pub fn get_root(&self, step: History<T>) -> Option<&NodeId> {
        self.trees.get(&step)
    }

    pub fn new_tree(&mut self, step: History<T>, ctx: ContextWrapper<T>) -> Result<NodeId, NodeId> {
        if let Some(&id) = self.trees.get(&step) {
            Err(id)
        } else {
            let node = HistoryNode {
                ctx,
                entry: step.clone(),
                children: HashMap::new(),
            };
            let id = self.arena.new_node(node);
            self.trees.insert(step, id);
            Ok(id)
        }
    }

    pub fn insert_tree<W, L, E, Wp>(
        &mut self,
        world: &W,
        hist: &Vec<History<T>>,
        startctx: &ContextWrapper<T>,
    ) where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency, LocId = L::LocId>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        let mut ctx = startctx.clone();
        if let Some(first) = hist.first() {
            ctx.replay(world, first);
            let mut node = match self.new_tree(first.clone(), ctx.clone()) {
                Ok(id) | Err(id) => id,
            };
            for step in hist.iter().skip(1) {
                ctx.replay(world, step);
                let child = match self.insert(node, step.clone(), ctx.clone()) {
                    Ok(id) | Err(id) => id,
                };
                node = child;
            }
        }
    }

    pub fn insert(
        &mut self,
        parent: NodeId,
        step: History<T>,
        ctx: ContextWrapper<T>,
    ) -> Result<NodeId, NodeId> {
        let parent_node = self.arena.get(parent).unwrap().get();
        if let Some(&id) = parent_node.children.get(&step) {
            Err(id)
        } else {
            let id = self.arena.new_node(HistoryNode {
                ctx,
                entry: step.clone(),
                children: HashMap::new(),
            });
            let parent_node = self.arena.get_mut(parent).unwrap().get_mut();
            parent_node.children.insert(step, id);
            parent.append(id, &mut self.arena);
            Ok(id)
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
