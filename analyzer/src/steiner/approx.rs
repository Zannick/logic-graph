use super::graph::SimpleGraph;
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
    /// Attempts to construct a Steiner approximation arborescence on the given graph,
    /// from the given root (index) and with the given required node (indices).
    fn compute(&self, root: V, required: HashSet<V, CommonHasher>) -> Option<ApproxSteiner<E>>;
    /// Same as compute but only returns the cost of the tree.
    fn compute_cost(&self, root: V, required: HashSet<V, CommonHasher>) -> Option<u64> {
        if let Some(ApproxSteiner {
            arborescence: _,
            cost,
        }) = self.compute(root, required)
        {
            Some(cost)
        } else {
            None
        }
    }
}
