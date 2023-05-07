#![allow(unused)]


use pheap::PairingHeap;

use super::approx::*;
use super::graph::*;
use crate::{new_hashset, new_hashmap, CommonHasher};
use std::fmt::Debug;
use std::collections::{HashSet, HashMap};

// Based on mouton5000's GFLAC3 algorithm

struct GFlac3<V, E> {
    graph: SimpleGraph<V, E>,
}

impl<V, E> GFlac3<V, E>
where
    V: Copy + Clone + Debug + Eq + PartialEq + std::hash::Hash,
    E: Copy + Clone + Eq + PartialEq + std::hash::Hash,
{
}

impl<V, E> SteinerAlgo<V, E> for GFlac3<V, E>
where
    V: Copy + Clone + Debug + Eq + PartialEq + std::hash::Hash,
    E: Copy + Clone + Eq + PartialEq + std::hash::Hash,
{
    const NAME: &'static str = "GFLAC3";

    fn from_graph(graph: SimpleGraph<V, E>) -> Self {
        Self { graph }
    }

    fn graph(&self) -> &SimpleGraph<V, E> {
        &self.graph
    }

    fn compute(
        &self,
        root: V,
        required: HashSet<V, CommonHasher>,
        extra_edges: &Vec<Edge<E>>,
    ) -> Option<ApproxSteiner<E>> {
        // things we need now:
        // edge costs map
        let mut edge_costs = Vec::new();
        edge_costs.extend(self.graph.edges.iter().map(|e| e.wt));
        edge_costs.extend(extra_edges.iter().map(|e| e.wt));
        // pairing heap per node
        let mut in_edges = Vec::new();
        in_edges.resize_with(self.graph.nodes.len(), || PairingHeap::new());

        for (ei, e) in self.graph.edges.iter().chain(extra_edges).enumerate() {
            in_edges[e.dst].insert(ei, e.wt);
        }

        // a set of visited nodes
        let mut visited = Vec::new();
        visited.resize(self.graph.nodes.len(), false);

        // a comparator? well, not really

        None
    }
}
