//! Implements the Steiner tree search algorithm based on shortest paths.

use super::approx::*;
use super::graph::*;
use crate::{new_hashset, CommonHasher};
use pheap::PairingHeap;
use std::collections::HashSet;
use std::fmt::Debug;

macro_rules! chain_index {
    ($list1:expr, $list2:expr, $index:expr) => {{
        if $index < $list1.len() {
            $list1[$index]
        } else {
            $list2[$index - $list1.len()]
        }
    }};
}

pub struct ShortestPaths<V, E> {
    graph: SimpleGraph<V, E>,

    // start vertex (index) -> end vertex (index) -> (list of edge indexes, total weight)
    paths: Vec<Vec<(Vec<usize>, Option<u64>)>>,
}

impl<V, E> SteinerAlgo<V, E> for ShortestPaths<V, E>
where
    V: Copy + Clone + Debug + Eq + PartialEq + std::hash::Hash,
    E: Copy + Clone + Debug + Eq + PartialEq + std::hash::Hash,
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
        // |E| is about 4-5x |V|; in dense graphs |E| would be prop. to |V| ** 2
        for start in 0..graph.nodes.len() {
            let mut present = Vec::new();
            present.resize(graph.nodes.len(), false);
            present[start] = true;

            // Heap elements are the edge index and the minimum time to
            // reach it plus the edge's weight
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
                        ph.insert(ei2, e2.wt + e.wt);
                    }
                }
            }
            while let Some((ei, prio)) = ph.delete_min() {
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
                            // Because prio is the minimum time to reach e and take it,
                            // and we haven't yet visited e.dst
                            // prio + e2.wt is the minimum time to reach e2 and take it
                            ph.insert(ei2, prio + e2.wt);
                        }
                    }
                }
            }
        }
        Self { graph, paths }
    }

    fn graph(&self) -> &SimpleGraph<V, E> {
        &self.graph
    }

    fn compute(
        &self,
        root: V,
        mut required: HashSet<V, CommonHasher>,
        extra_edges: &Vec<Edge<E>>,
    ) -> Option<ApproxSteiner<E>> {
        let root_index = self.graph.node_index_map[&root];

        let mut nodes = vec![root_index];
        let mut edges = new_hashset();
        let mut cost = 0;

        // We don't actually need a new paths table, since we require that all
        // extra edges originate from the root. Therefore, we can just get the shortest
        // path among: a) the original shortest paths, b) any new edge plus its shortest path

        while !required.is_empty() {
            let mut min = None;
            let mut newpath_holder = Vec::new();

            // Find the minimum path from any node we have to any required node
            // New edges can find a better route.
            for (new_ei, e) in extra_edges.iter().enumerate() {
                // But if we have already added that node, it's not the shortest path!
                if !nodes.contains(&e.dst) {
                    for req in required.iter() {
                        let ri = self.graph.node_index_map[req];
                        if let Some(t) = self.paths[e.dst][ri].1 {
                            let time = e.wt + t;
                            if let Some((_, old_best, _, _)) = min {
                                if time < old_best {
                                    // The "index" of the new edge is its real index plus |E|
                                    let mut path = vec![self.graph.edges.len() + new_ei];
                                    path.extend(&self.paths[e.dst][ri].0);
                                    newpath_holder.push(path);
                                    min = Some((newpath_holder.last().unwrap(), time, *req, root_index));
                                }
                            } else {
                                let mut path = vec![self.graph.edges.len() + new_ei];
                                path.extend(&self.paths[e.dst][ri].0);
                                newpath_holder.push(path);
                                min = Some((newpath_holder.last().unwrap(), time, *req, root_index));
                            }
                        }
                    }
                }
            }
            for &start in nodes.iter() {
                for req in required.iter() {
                    let ri = self.graph.node_index_map[req];
                    if let Some(t) = self.paths[start][ri].1 {
                        if let Some((_, old_best, _, _)) = min {
                            if t < old_best {
                                min = Some((&self.paths[start][ri].0, t, *req, start));
                            }
                        } else {
                            min = Some((&self.paths[start][ri].0, t, *req, start));
                        }
                    }
                }
            }
            if let Some((path, best, req, _)) = min {
                required.remove(&req);
                // Because the graph has no negative edges,
                // the minimum path must have no intermediate nodes or edges already in the tree
                nodes.extend(
                    path
                        .iter()
                        .map(|&ei| chain_index!(self.graph.edges, extra_edges, ei).dst),
                );
                edges.extend(
                    path
                        .iter()
                        .map(|&ei| chain_index!(self.graph.edges, extra_edges, ei).id),
                );
                cost += best;
            } else {
                // There is a location we cannot access. This is a failure.
                return None;
            }
        }

        if edges.is_empty() {
            None
        } else {
            Some(ApproxSteiner {
                arborescence: edges,
                cost,
            })
        }
    }
}
