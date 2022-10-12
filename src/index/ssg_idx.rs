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
            root_nodes: Vec::new(),
            width: 0,
            index_size: params.index_size,
            angle: params.angle,
            threshold: (params.angle / E::from_f32(180.0).unwrap() * E::PI()).cos(),
            root_size: params.root_size,

            search_times: 0,
        }
    }

    fn build_knn_graph(&mut self) {
        let tmp_graph = Arc::new(Mutex::new(vec![vec![0]; self.nodes.len()]));
        vec_iter!(self.nodes, ctr);
        ctr.zip(0..self.nodes.len()).for_each(|(item, n)| {
            let mut heap = BinaryHeap::with_capacity(self.init_k);
            self.nodes
                .iter()
                .zip(0..self.nodes.len())
                .for_each(|(node, i)| {
                    if i == n {
                        return;
                    }
                    heap.push(neighbor::Neighbor::new(
                        i,
                        item.metric(node, self.mt).unwrap(),
                    ));
                    if heap.len() > self.init_k {
                        heap.pop();
                    }
                });
            let mut tmp = Vec::with_capacity(heap.len());
            while !heap.is_empty() {
                tmp.push(heap.pop().unwrap().idx());
            }

            tmp_graph.lock().unwrap()[n] = tmp;
        });
        self.graph = tmp_graph.lock().unwrap().to_vec();
        self.knn_graph = tmp_graph.lock().unwrap().to_vec();
    }

    fn get_random_nodes_idx_lite(&self, indices: &mut [usize]) {
        let mut rng = rand::thread_rng();
        (0..indices.len()).for_each(|i| {
            indices[i] = rng.gen_range(0..self.nodes.len() - indices.len());
        });
    }

    fn get_point_neighbor_size_neighbors(
        &self,
        q: usize,
        expand_neighbors_tmp: &mut Vec<neighbor::Neighbor<E, usize>>,
    ) {
        let mut flags = HashSet::with_capacity(self.neighbor_neighbor_size);

        flags.insert(q);
        for neighbor_id in self.graph[q].iter() {
            for nn_id in self.graph[*neighbor_id].iter() {
                if *neighbor_id == *nn_id {
                    continue;
                }
                if flags.contains(nn_id) {
                    continue;
                }
                flags.insert(*nn_id);
                let dist = self.nodes[q].metric(&self.nodes[*nn_id], self.mt).unwrap();
                expand_neighbors_tmp.push(neighbor::Neighbor::new(*nn_id, dist));
                if expand_neighbors_tmp.len() >= self.neighbor_neighbor_size {
                    return;
                }
            }
        }
    }

    fn expand_connectivity(&mut self) {
        let range = self.index_size;

        let mut ids: Vec<usize> = (0..self.nodes.len()).collect();
        ids.shuffle(&mut thread_rng());
        for id in ids.iter().take(self.root_size) {
            self.root_nodes.push(*id);
        }

        (0..self.root_size).for_each(|i| {
            let root_id = self.root_nodes[i];
            let mut flags = HashSet::new();
            let mut my_queue = VecDeque::new();
            my_queue.push_back(root_id);
            flags.insert(root_id);

            let mut unknown_set: Vec<usize> = Vec::with_capacity(1);
            while !unknown_set.is_empty() {
                while !my_queue.is_empty() {
                    let q_front = my_queue.pop_front().unwrap();

                    for j in 0..self.graph[q_front].len() {
                        let child = self.graph[q_front][j];
                        if flags.contains(&child) {
                            continue;
                        }
                        flags.insert(child);
                        my_queue.push_back(child);
                    }
                }
                unknown_set.clear();
                for j in 0..self.nodes.len() {
                    if flags.contains(&j) {
                        continue;
                    }
                    unknown_set.push(j);
                }
                if !unknown_set.is_empty() {
                    for j in 0..self.nodes.len() {
                        if flags.contains(&j) && self.graph[j].len() < range {
                            self.graph[j].push(unknown_set[0]);
                            break;
                        }
                    }
                    my_queue.push_back(unknown_set[0]);
                    flags.insert(unknown_set[0]);
                }
            }
        });
    }

    fn link_each_nodes(&mut self, pruned_graph_tmp: &mut Vec<neighbor::Neighbor<E, usize>>) {
        let mut expand_neighbors_tmp = Vec::new();
        (0..self.nodes.len()).for_each(|i| {
            expand_neighbors_tmp.clear();
            self.get_point_neighbor_size_neighbors(i, &mut expand_neighbors_tmp); // get related one
            self.prune_graph(
                i,
                &mut expand_neighbors_tmp,
                self.threshold,
                pruned_graph_tmp,
            );
        });
        (0..self.nodes.len()).for_each(|i| {
            self.inter_insert(i, self.index_size, pruned_graph_tmp);
        });
    }

    fn prune_graph(
        &mut self,
        query_id: usize,
        expand_neighbors_tmp: &mut Vec<neighbor::Neighbor<E, usize>>,
        threshold: E,
        pruned_graph_tmp: &mut Vec<neighbor::Neighbor<E, usize>>,
    ) {
        let mut start = 0;
        let mut flags = HashSet::with_capacity(expand_neighbors_tmp.len());
        for iter in expand_neighbors_tmp.iter() {
            flags.insert(iter.idx());
        }
        self.graph[query_id].iter().for_each(|linked_id| {
            if flags.contains(linked_id) {
                return;
            }
            expand_neighbors_tmp.push(neighbor::Neighbor::new(
                *linked_id,
                self.nodes[query_id]
                    .metric(&self.nodes[*linked_id], self.mt)
                    .unwrap(),
            ));
        });

        expand_neighbors_tmp.sort_unstable();
        let mut result = Vec::new();
        if expand_neighbors_tmp[start].idx() == query_id {
            start += 1;
        }
        result.push(expand_neighbors_tmp[start].clone());

        start += 1;
        while result.len() < self.index_size && start < expand_neighbors_tmp.len() {
            let p = &expand_neighbors_tmp[start];
            let mut occlude = false;
            // TODO: check every metrics, and decide use euclidean forcibly.
            for iter in result.iter() {
                if p.idx() == iter.idx() {
                    // stop early
                    occlude = true;
                    break;
                }
                let djk = self.nodes[iter.idx()]
                    .metric(&self.nodes[p.idx()], self.mt)
                    .unwrap();
                let cos_ij = (p.distance().powi(2) + iter.distance().powi(2) - djk.powi(2))
                    / (E::from_usize(2).unwrap() * (p.distance() * iter.distance()));

                if cos_ij > threshold {
                    occlude = true;
                    break;
                }
            }
            if !occlude {
                result.push(p.clone());
            }
            start += 1;
        }

        (0..result.len()).for_each(|t| {
            pruned_graph_tmp[t + query_id * self.index_size]._idx = result[t].idx();
            pruned_graph_tmp[t + query_id * self.index_size]._distance = result[t].distance();
        });
        if resul