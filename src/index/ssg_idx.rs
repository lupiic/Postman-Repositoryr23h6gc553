#![allow(dead_code)]
use crate::core::ann_index;
use crate::core::kmeans;
use crate::core::metrics;
use crate::core::neighbor;
use crate::core::node;
use crate::index::ssg_params::SSGParams;
use crate::vec_iter;
use fixedbitset::FixedBitSet;
use rand::prelude::*;
#[cfg(not(feature = "no_thread"))]
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::collections::VecDeque;

use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
pub struct SSGIndex<E: node::FloatElement, T: node::IdxType> {
    #[serde(skip_serializing, skip_deserializing)]
    nodes: Vec<Box<node::Node<E, T>>>,
    tmp_nodes: Vec<node::Node<E, T>>, // only use for serialization scene
    mt: metrics::Metric,
    dimension: usize,
    neighbor_neighbor_size: usize,
    index_size: usize,
    graph: Vec<Vec<usize>>,
    knn_graph: Vec<Vec<usize>>,
    init_k: usize, // as knn's k
    root_nodes: Vec<usize>,
    width: usize,
    angle: E,
    threshold: E,
    root_size: usize,

    // stat
    search_times: usize,
}

impl<E: node::FloatElement, T: node::IdxType> SSGIndex<E, T> {
    pub fn new(dimension: usize, params: &SSGParams<E>) -> SSGIndex<E, T> {
        SSGIndex::<E, T> {
            nodes: Vec::new(),
            tmp_nodes: Vec::new(),
            mt: metrics::Metric::Unknown,
            dimension,
            neighbor_neighbor_size: params.neighbor_neighbor_size,
            init_k: params.init_k,
            graph: Vec::new(),
            knn_graph: Vec::new(),
            root_n