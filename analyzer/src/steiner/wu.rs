#![allow(unused)]

//! Implements the algorithm from Wu et al 1986, building the tree upward
//! from the terminal nodes.

use super::approx::*;
use super::graph::*;
use crate::new_hashmap;
use crate::{new_hashset, CommonHasher};
use disjoint_hash_set::DisjointHashSet;
use pheap::PairingHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use union_find::*;

struct Wu<V, E> {
    // Stored for simple reconstruction from a different root
    graph: SimpleGraph<V, E>,

    // These will be recreated for each computed tree

    // Queues for each tree, will be removed as they are merged
    edge_queues: HashMap<usize, PairingHeap<Edge<E>, u64>, CommonHasher>,
    // collected edges
    edges: HashSet<E, CommonHasher>,
    // collected weight
    cost: u64,
    // trees
    trees: DisjointHashSet<usize>,
    tree_count: usize,
}

impl<V, E> Wu<V, E>
where
    V: Copy + Clone,
    E: Copy + Clone + Eq + PartialEq + std::hash::Hash,
{
    fn rebuild(&mut self, root: usize, targets: &HashSet<usize, CommonHasher>) {
        self.edges.clear();
        self.cost = 0;
        self.trees = DisjointHashSet::new();
        for target in targets {
            self.trees.insert(*target);
        }
        self.tree_count = targets.len();
        self.edge_queues.clear();
        for (idx, _) in self.graph.nodes.iter().enumerate() {
            self.edge_queues.insert(idx, PairingHeap::new());
        }
        for edge in self.graph.edges.as_slice() {
            if edge.dst != root {
                self.edge_queues
                    .get_mut(&edge.dst)
                    .unwrap()
                    .insert(*edge, edge.wt);
            }
        }
    }

    fn recompute(&mut self, root: usize, required: HashSet<usize, CommonHasher>) -> bool {
        self.rebuild(root, &required);
        let mut total_heap = PairingHeap::new();
        for t in required {
            total_heap = total_heap.merge(self.edge_queues.remove(&t).unwrap());
        }
        while self.tree_count > 1 && !self.trees.contains(root) {
            if let Some((e, _)) = total_heap.delete_min() {
                if self.trees.is_linked(e.src, e.dst) {
                    continue;
                }

                // if the src is already tracked in our trees
                // then we already have its incoming edges
                // also we reduce the tree count from the merge
                if self.trees.contains(e.src) {
                    self.tree_count -= 1;
                } else {
                    // otherwise we have new edges to potentially use
                    total_heap = total_heap.merge(self.edge_queues.remove(&e.src).unwrap());
                }
                self.trees.link(e.src, e.dst);
                self.edges.insert(e.id);
                self.cost += e.wt;
            } else {
                return false;
            }
        }
        true
    }
}

impl<V, E> SteinerAlgo<V, E> for Wu<V, E>
where
    V: Copy + Clone,
    E: Copy + Clone + Eq + PartialEq + std::hash::Hash,
{
    fn from_graph(graph: &SimpleGraph<V, E>) -> Self {
        Self {
            graph: graph.clone(),
            edge_queues: new_hashmap(),
            edges: new_hashset(),
            cost: 0,
            trees: DisjointHashSet::new(),
            tree_count: 0,
        }
    }

    fn compute(
        &mut self,
        root: usize,
        required: HashSet<usize, CommonHasher>,
    ) -> Option<ApproxSteiner<E>> {
        if self.recompute(root, required) {
            Some(ApproxSteiner {
                arborescence: self.edges.clone(),
                cost: self.cost,
            })
        } else {
            None
        }
    }

    fn compute_cost(&mut self, root: usize, required: HashSet<usize, CommonHasher>) -> Option<u64> {
        if self.recompute(root, required) {
            Some(self.cost)
        } else {
            None
        }
    }
}
