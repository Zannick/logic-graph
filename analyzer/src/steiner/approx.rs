use super::graph::{Edge, SimpleGraph};
use crate::CommonHasher;
use std::collections::HashSet;
use std::fmt::Display;

#[derive(Debug)]
pub struct ApproxSteiner<E> {
    /// The set of the edge IDs that are in the arborescence.
    pub arborescence: HashSet<E, CommonHasher>,
    /// Returns the total cost of the edges in the arborescence.
    pub cost: u64,
}

impl<E> Display for ApproxSteiner<E>
where
    E: Copy + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut edges = self
            .arborescence
            .iter()
            .map(|e| format!("{}", e))
            .collect::<Vec<_>>();
        edges.sort_unstable();
        write!(
            f,
            "Approximate steiner tree: edges={}, cost={}\n{}",
            edges.len(),
            self.cost,
            edges.join("\n")
        )
    }
}

pub trait SteinerAlgo<V, E> {
    const NAME: &'static str;
    fn from_graph(graph: SimpleGraph<V, E>) -> Self;
    fn graph(&self) -> &SimpleGraph<V, E>;

    /// Attempts to construct a Steiner approximation arborescence on the given graph,
    /// from the given root (index) and with the given required node (indices).
    /// Optionally, a list of node subsets can be given, each with a nonzero number of necessary nodes
    /// in that set; once that many of those nodes are visited, the rest are no longer required.
    /// Items in subsets that aren't in the full required list will be ignored.
    fn compute(
        &self,
        root: V,
        required: HashSet<V, CommonHasher>,
        subsets: Vec<(HashSet<V, CommonHasher>, i16)>,
        extra_edges: &Vec<Edge<E>>,
    ) -> Option<ApproxSteiner<E>>;
    /// Same as compute but only returns the cost of the tree.
    fn compute_cost(
        &self,
        root: V,
        required: HashSet<V, CommonHasher>,
        subsets: Vec<(HashSet<V, CommonHasher>, i16)>,
        extra_edges: &Vec<Edge<E>>,
    ) -> Option<u64> {
        if let Some(ApproxSteiner {
            arborescence: _,
            cost,
        }) = self.compute(root, required, subsets, extra_edges)
        {
            Some(cost)
        } else {
            None
        }
    }
}
