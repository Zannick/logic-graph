#![allow(unused)]

use std::collections::HashSet;

use super::approx::*;
use super::graph::*;
use crate::{new_hashset, CommonHasher};

// Based on mouton5000's GFLAC3 algorithm

struct GFlac3 {
    /// edge weights while running
    costs: Vec<usize>,
    /// required nodes
    req_nodes: HashSet<usize, CommonHasher>,

}
