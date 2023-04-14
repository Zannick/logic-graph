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

pub struct Wu<V, E> {
    // Stored for simple reconstruction from a different root
    graph: SimpleGraph<V, E>,
}

impl<V, E> SteinerAlgo<V, E> for Wu<V, E>
where
    V: Copy + Clone + Eq + PartialEq + std::hash::Hash,
    E: Copy + Clone + Eq + PartialEq + std::hash::Hash,
{
    const NAME: &'static str = "Wu";

    fn from_graph(graph: SimpleGraph<V, E>) -> Self {
        Self {
            graph,
        }
    }

    fn compute(&self, root: V, required: HashSet<V, CommonHasher>) -> Option<ApproxSteiner<E>> {
        // Queues for each tree, will be removed as they are merged
        let mut edge_queues: HashMap<usize, PairingHeap<Edge<E>, u64>, CommonHasher> = new_hashmap();
        // collected edges
        let mut edges: HashSet<E, CommonHasher> = new_hashset();
        // collected weight
        let mut cost: u64 = 0;
        // trees
        let mut trees: DisjointHashSet<usize> = DisjointHashSet::new();
        let mut tree_count = required.len();

        // Setup
        for target in &required {
            trees.insert(self.graph.node_index_map[target]);
        }
        for (idx, _) in self.graph.nodes.iter().enumerate() {
            edge_queues.insert(idx, PairingHeap::new());
        }
        let root_index = self.graph.node_index_map[&root];
        for edge in self.graph.edges.as_slice() {
            if edge.dst != root_index {
                edge_queues
                    .get_mut(&edge.dst)
                    .unwrap()
                    .insert(*edge, edge.wt);
            }
        }

        // Run algorithm
        let mut total_heap = PairingHeap::new();
        for t in required {
            total_heap = total_heap.merge(
                edge_queues
                    .remove(&self.graph.node_index_map[&t])
                    .unwrap(),
            );
        }
        let root_index = self.graph.node_index_map[&root];
        while tree_count > 1 && !trees.contains(root_index) {
            // Takes the minimum edge that connects two disjoint trees, or that adds a node
            // not yet in a tree to a tree.
            // This is not very good even greedy, since it will take a longer
            // path made of short steps... and may not even produce a tree rooted at root
            if let Some((e, _)) = total_heap.delete_min() {
                if trees.is_linked(e.src, e.dst) {
                    continue;
                }

                // if the src is already tracked in our trees
                // then we already have its incoming edges
                // also we reduce the tree count from the merge
                if trees.contains(e.src) {
                    tree_count -= 1;
                } else {
                    // otherwise we have new edges to potentially use
                    total_heap = total_heap.merge(edge_queues.remove(&e.src).unwrap());
                }
                trees.link(e.src, e.dst);
                edges.insert(e.id);
                cost += e.wt;
            } else {
                return None;
            }
        }
        
        Some(ApproxSteiner {
            arborescence: edges,
            cost,
        })
    }
}
