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
    trees: Vec<NodeId>,
}

impl<T> HistoryTree<T>
where
    T: Ctx,
{
    pub fn new() -> HistoryTree<T> {
        HistoryTree {
            arena: Arena::new(),
            trees: Vec::new(),
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

    // This is pretty dangerous to do on anything other than a leaf.
    pub fn get_mut(&self, id: NodeId) -> Option<&mut ContextWrapper<T>> {
        if id.children(&self.arena).next() == None {
            None
        } else {
            Some(&mut self.arena.get_mut(id).unwrap().get_mut().ctx)
        }
    }

    pub fn new_tree(&mut self, step: History<T>, ctx: ContextWrapper<T>) -> NodeId {
        let node = HistoryNode {
            ctx,
            entry: step,
            children: HashMap::new(),
        };
        let id = self.arena.new_node(node);
        self.trees.push(id);
        id
    }

    pub fn insert_tree<W, L, E, Wp>(
        &mut self,
        world: &W,
        hist: &Vec<History<T>>,
        startctx: &ContextWrapper<T>,
    ) -> (NodeId, NodeId)
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency, LocId = L::LocId>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        let mut ctx = startctx.clone();
        assert!(Some(&ctx.last) == hist.first());
        let root = self.new_tree(ctx.last, ctx.clone());

        (root, self.insert_tree_from(world, hist, root))
    }

    pub fn insert_tree_from<W, L, E, Wp>(
        &mut self,
        world: &W,
        hist: &Vec<History<T>>,
        root: NodeId,
    ) -> NodeId
    where
        W: World<Location = L, Exit = E, Warp = Wp>,
        L: Location<Context = T>,
        T: Ctx<World = W>,
        E: Exit<Context = T, Currency = <L as Accessible>::Currency, LocId = L::LocId>,
        Wp: Warp<SpotId = <E as Exit>::SpotId, Context = T, Currency = <L as Accessible>::Currency>,
    {
        let mut ctx = self.get(root).clone();
        let node = root;
        for step in hist.iter().skip(1) {
            ctx.replay(world, step);
            let child = match self.insert(node, *step, ctx.clone()) {
                Ok(id) | Err(id) => id,
            };
            node = child;
        }
        node
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
                entry: step,
                children: HashMap::new(),
            });
            let parent_node = self.arena.get_mut(parent).unwrap().get_mut();
            parent_node.children.insert(step, id);
            parent.append(id, &mut self.arena);
            Ok(id)
        }
    }

    pub fn insert_and_get(
        &mut self,
        parent: NodeId,
        step: History<T>,
        ctx: ContextWrapper<T>,
    ) -> NodeId {
        match self.insert(parent, step, ctx) {
            Ok(id) | Err(id) => id,
        }
    }

    pub fn rev_history(&self, id: NodeId) -> impl Iterator<Item = History<T>> + '_ {
        id.ancestors(&self.arena)
            .map(|n| self.arena.get(n).unwrap().get().entry)
    }

    pub fn get_history(&self, id: NodeId) -> Vec<History<T>> {
        let mut vec: Vec<History<T>> = id
            .ancestors(&self.arena)
            .map(|n| self.arena.get(n).unwrap().get().entry)
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
