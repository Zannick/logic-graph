#![allow(unused)]

use log;
use std::collections::HashMap;
use std::time::Instant;

use pheap::PairingHeap;
use union_find::*;

use crate::context::*;
use crate::new_hashmap;
use crate::new_hashset;
use crate::world::*;
use crate::CommonHasher;

// This struct is more for the MDST algorithm...

pub struct Node<V, E> {
    // generally the external id
    id: V,
    // an index into the edges list
    edge_in: Option<usize>,
    // A constant subtracted from weight
    constant: u64,
    // an index into the nodes or supernodes list
    prev: Option<usize>,
    // parent
    // child
    queue: PairingHeap<Edge<E>, u64>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Edge<E> {
    // generally the external id
    pub id: E,
    pub src: usize,
    pub dst: usize,
    pub wt: u64,
}

pub struct Graph<V, E> {
    nodes: Vec<Node<V, E>>,
    union: QuickFindUf<UnionBySize>,
}
impl<V, E> Graph<V, E> {}

#[derive(Clone)]
pub struct SimpleGraph<V, E> {
    pub(crate) nodes: Vec<V>,
    pub(crate) node_index_map: HashMap<V, usize, CommonHasher>,
    pub(crate) edges: Vec<Edge<E>>,
}

// analyzer-specific stuff

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExternalNodeId<S, C> {
    Spot(S),
    Canon(C),
}
pub type NodeId<W> = ExternalNodeId<
    <<W as World>::Exit as Exit>::SpotId,
    <<W as World>::Location as Location>::CanonId,
>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExternalEdgeId<S, C, Wp, A> {
    Spots(S, S),
    Canon(S, C),
    Warp(Wp),
    Action(A),
}
pub type EdgeId<W> = ExternalEdgeId<
    <<W as World>::Exit as Exit>::SpotId,
    <<W as World>::Location as Location>::CanonId,
    <<W as World>::Warp as Warp>::WarpId,
    <<W as World>::Action as Action>::ActionId,
>;

pub fn build_graph<W, T>(world: &W, startctx: &T) -> Graph<NodeId<W>, EdgeId<W>>
where
    W: World,
    T: Ctx<World = W>,
{
    let mut nodes = Vec::new();
    nodes.extend(
        world
            .get_all_spots()
            .iter()
            .map(|s| ExternalNodeId::Spot(*s)),
    );
    let mut canon = new_hashset();
    nodes.extend(world.get_all_locations().iter().filter_map(|loc| {
        if startctx.todo(loc.id()) {
            if !canon.contains(&loc.canon_id()) {
                canon.insert(loc.canon_id());
                Some(ExternalNodeId::Canon(loc.canon_id()))
            } else {
                None
            }
        } else {
            None
        }
    }));
    let mut nodes: Vec<_> = nodes
        .into_iter()
        .map(|id| Node {
            id,
            edge_in: None,
            constant: 0,
            prev: None,
            queue: PairingHeap::new(),
        })
        .collect();
    let mut node_index_map = new_hashmap();
    for (index, n) in nodes.iter().enumerate() {
        node_index_map.insert(n.id, index);
    }

    for (s, t, dist) in world.base_edges().into_iter() {
        nodes[node_index_map[&ExternalNodeId::Spot(t)]]
            .queue
            .insert(
                Edge {
                    id: ExternalEdgeId::Spots(s, t),
                    src: node_index_map[&ExternalNodeId::Spot(s)],
                    dst: node_index_map[&ExternalNodeId::Spot(t)],
                    wt: dist.into(),
                },
                dist.into(),
            );
    }
    for loc in world.get_all_locations() {
        if startctx.todo(loc.id()) {
            let s = world.get_location_spot(loc.id());
            let t = ExternalNodeId::Canon(loc.canon_id());
            let id = ExternalEdgeId::Canon(s, loc.canon_id());
            let wt = loc.base_time().try_into().unwrap();
            nodes[node_index_map[&t]].queue.insert(
                Edge {
                    id,
                    src: node_index_map[&ExternalNodeId::Spot(s)],
                    dst: node_index_map[&t],
                    wt,
                },
                wt,
            );
        }
    }
    let union = QuickFindUf::new(nodes.len());
    Graph { nodes, union }
}

pub fn build_simple_graph<W, T>(world: &W, startctx: &T) -> SimpleGraph<NodeId<W>, EdgeId<W>>
where
    W: World,
    T: Ctx<World = W>,
{
    let s = Instant::now();
    let mut nodes = Vec::new();
    // 3 types of nodes: spots, locations, canon locations
    nodes.extend(
        world
            .get_all_spots()
            .iter()
            .map(|s| ExternalNodeId::Spot(*s)),
    );
    let mut canon = new_hashset();
    nodes.extend(world.get_all_locations().iter().filter_map(|loc| {
        if startctx.todo(loc.id()) {
            if !canon.contains(&loc.canon_id()) {
                canon.insert(loc.canon_id());
                Some(ExternalNodeId::Canon(loc.canon_id()))
            } else {
                None
            }
        } else {
            None
        }
    }));

    // Edges use the indices of nodes, so we need a map
    let mut node_index_map = new_hashmap();
    for (index, n) in nodes.iter().enumerate() {
        node_index_map.insert(*n, index);
    }

    // Two types of edges: the spot -> spot connections from base_edges,
    // and spot -> location/canon nodes
    let mut edges = Vec::new();
    for (s, t, dist) in world.base_edges().into_iter() {
        edges.push(Edge {
            id: ExternalEdgeId::Spots(s, t),
            src: node_index_map[&ExternalNodeId::Spot(s)],
            dst: node_index_map[&ExternalNodeId::Spot(t)],
            wt: dist.into(),
        });
    }

    for loc in world.get_all_locations() {
        if startctx.todo(loc.id()) {
            let s = world.get_location_spot(loc.id());
            let t = ExternalNodeId::Canon(loc.canon_id());
            let id = ExternalEdgeId::Canon(s, loc.canon_id());
            let wt = loc.base_time().try_into().unwrap();
            edges.push(Edge {
                id,
                src: node_index_map[&ExternalNodeId::Spot(s)],
                dst: node_index_map[&t],
                wt,
            });
        }
    }
    log::info!(
        "Built simple graph of {} nodes and {} edges in {:?}",
        nodes.len(),
        edges.len(),
        s.elapsed()
    );
    SimpleGraph {
        nodes,
        node_index_map,
        edges,
    }
}

impl<V, E> SimpleGraph<V, E>
where
    V: Eq + std::hash::Hash,
{
    pub fn new_edge(&self, id: E, src: V, dst: V, wt: u64) -> Edge<E> {
        Edge {
            id,
            src: self.node_index_map[&src],
            dst: self.node_index_map[&dst],
            wt,
        }
    }
}

pub fn spot_to_graph_node<W, E>(spot_id: E::SpotId) -> NodeId<W>
where
    W: World<Exit = E>,
    E: Exit,
{
    ExternalNodeId::Spot(spot_id)
}

pub fn loc_to_graph_node<W, L>(world: &W, loc_id: L::LocId) -> NodeId<W>
where
    W: World<Location = L>,
    L: Location,
{
    let loc = world.get_location(loc_id);
    ExternalNodeId::Canon(loc.canon_id())
}
