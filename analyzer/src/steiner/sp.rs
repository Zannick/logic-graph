//! Implements the Steiner tree search algorithm based on shortest paths.

use super::approx::*;
use super::graph::*;
use crate::{new_hashset, CommonHasher};
use pheap::PairingHeap;
use std::collections::HashSet;
use std::fmt::Debug;

pub struct ShortestPaths<V, E> {
    graph: SimpleGraph<V, E>,

    // start vertex (index) -> end vertex (index) -> (list of edge indexes, total weight)
    paths: Vec<Vec<(Vec<usize>, Option<u64>)>>,

    // collected nodes
    nodes: Vec<usize>,
    // collected edges
    edges: HashSet<E, CommonHasher>,
    // collected weight
    cost: u64,
}

impl<V, E> ShortestPaths<V, E>
where
    V: Copy + Clone + Debug + Eq + PartialEq + std::hash::Hash,
    E: Copy + Clone + Eq + PartialEq + std::hash::Hash,
{
    fn reset(&mut self, root_index: usize) {
        self.nodes.clear();
        self.nodes.push(root_index);
        self.edges.clear();
        self.cost = 0;
    }

    fn recompute(&mut self, root: V, mut required: HashSet<V, CommonHasher>) -> bool {
        let root_index = self.graph.node_index_map[&root];
        self.reset(root_index);

        while !required.is_empty() {
            let r = required.iter().next().unwrap();
            let ri = self.graph.node_index_map[r];

            // Find the minimum path from any node we have to any required node
            let mut min = if let Some(rt) = self.paths[root_index][ri].1 {
                (&self.paths[root_index][ri].0, rt, *r, root_index)
            } else {
                println!(
                    "No path from {:?} to {:?}",
                    self.graph.nodes[root_index], self.graph.nodes[ri],
                );
                required.remove(&r.clone());
                continue;
            };
            for &start in self.nodes.iter() {
                for req in required.iter() {
                    let ri = self.graph.node_index_map[req];
                    if let Some(t) = self.paths[start][ri].1 {
                        if t < min.1 {
                            min = (&self.paths[start][ri].0, t, *req, start);
                        }
                    }
                }
            }
            required.remove(&min.2);
            // Because the graph has no negative edges,
            // the minimum path must have no intermediate nodes or edges already in the tree
            self.nodes
                .extend(min.0.iter().map(|&ei| self.graph.edges[ei].dst));
            self.edges
                .extend(min.0.iter().map(|&ei| self.graph.edges[ei].id));
            self.cost += min.1;
            /*
            println!(
                "Adding path {:?} -> {:?} to arborescence (cost: {})",
                self.graph.nodes[min.3], min.2, min.1
            );
            */
        }

        true
    }
}

impl<V, E> SteinerAlgo<V, E> for ShortestPaths<V, E>
where
    V: Copy + Clone + Debug + Eq + PartialEq + std::hash::Hash,
    E: Copy + Clone + Eq + PartialEq + std::hash::Hash,
{
    const NAME: &'static str = "ShortestPaths";

    fn from_graph(graph: SimpleGraph<V, E>) -> Self {
        let mut paths = Vec::new();
        paths.resize_with(graph.nodes.len(), || {
            let mut v = Vec::new();
            v.resize_with(graph.nodes.len(), || (Vec::new(), None));
            v
        });
        for i in 0..graph.nodes.len() {
            paths[i][i].1 = Some(0u64);
        }

        let mut edges_by_start = Vec::new();
        edges_by_start.resize_with(graph.nodes.len(), Vec::new);

        for (i, e) in graph.edges.iter().enumerate() {
            edges_by_start[e.src].push(i);
        }

        // Dijkstra's is better for sparse graphs
        for start in 0..graph.nodes.len() {
            let mut present = Vec::new();
            present.resize(graph.nodes.len(), false);
            present[start] = true;

            let mut ph = PairingHeap::new();
            for &ei in &edges_by_start[start] {
                let e = &graph.edges[ei];
                if let Some(k) = paths[start][e.dst].1 {
                    if e.wt < k {
                        paths[start][e.dst].0.clear();
                        paths[start][e.dst].0.push(ei);
                        paths[start][e.dst].1 = Some(e.wt);
                    }
                } else {
                    paths[start][e.dst].0.push(ei);
                    paths[start][e.dst].1 = Some(e.wt);
                };
                if !present[e.dst] {
                    present[e.dst] = true;
                    // Immediately do the second-order edges, rather than insert the first edges
                    for &ei2 in &edges_by_start[e.dst] {
                        let e2 = &graph.edges[ei2];
                        ph.insert(ei2, e2.wt);
                    }
                }
            }
            while let Some((ei, _)) = ph.delete_min() {
                let e = &graph.edges[ei];
                // should always be true
                if let (v, Some(w)) = &paths[start][e.src] {
                    let w_new = *w + e.wt;
                    if let Some(w_old) = paths[start][e.dst].1 {
                        if w_new < w_old {
                            let mut path = v.clone();
                            path.push(ei);
                            paths[start][e.dst] = (path, Some(w_new));
                        }
                    } else {
                        let mut path = v.clone();
                        path.push(ei);
                        paths[start][e.dst] = (path, Some(w_new));
                    }

                    if !present[e.dst] {
                        present[e.dst] = true;
                        for &ei2 in &edges_by_start[e.dst] {
                            let e2 = &graph.edges[ei2];
                            ph.insert(ei2, e2.wt);
                        }
                    }
                }
            }
        }
        Self {
            graph,
            paths,
            nodes: Vec::new(),
            edges: new_hashset(),
            cost: 0,
        }
    }

    fn compute(&mut self, root: V, required: HashSet<V, CommonHasher>) -> Option<ApproxSteiner<E>> {
        if self.recompute(root, required) {
            Some(ApproxSteiner {
                arborescence: self.edges.clone(),
                cost: self.cost,
            })
        } else {
            None
        }
    }

    fn compute_cost(&mut self, root: V, required: HashSet<V, CommonHasher>) -> Option<u64> {
        if self.recompute(root, required) {
            Some(self.cost)
        } else {
            None
        }
    }
}
