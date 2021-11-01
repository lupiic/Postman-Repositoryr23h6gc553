
#![allow(dead_code)]
use crate::core::metrics;
use crate::core::neighbor::Neighbor;
use crate::core::node::{FloatElement, IdxType, Node};
use fixedbitset::FixedBitSet;
use rand::seq::SliceRandom;
use rand::Rng;
use rayon::prelude::*;
use std::collections::BinaryHeap;
use std::sync::mpsc;

use std::sync::{Arc, Mutex};

pub fn naive_build_knn_graph<E: FloatElement, T: IdxType>(
    nodes: &[Box<Node<E, T>>],
    mt: metrics::Metric,
    k: usize,
    graph: &mut Vec<Vec<Neighbor<E, usize>>>, // TODO: not use this one
) {
    let tmp_graph = Arc::new(Mutex::new(graph));
    (0..nodes.len()).into_par_iter().for_each(|n| {
        let item = &nodes[n];
        let mut heap = BinaryHeap::with_capacity(k);
        (0..nodes.len()).for_each(|i| {
            if i == n {
                return;
            }
            heap.push(Neighbor::new(i, item.metric(&nodes[i], mt).unwrap()));
            if heap.len() > k {
                heap.pop();
            }
        });
        let mut tmp = Vec::with_capacity(heap.len());
        while !heap.is_empty() {
            tmp.push(heap.pop().unwrap());
        }

        tmp_graph.lock().unwrap()[n].clear();
        tmp_graph.lock().unwrap()[n] = tmp;
    });
}

pub struct NNDescentHandler<'a, E: FloatElement, T: IdxType> {
    nodes: &'a [Box<Node<E, T>>],
    graph: Vec<Arc<Mutex<BinaryHeap<Neighbor<E, usize>>>>>,
    mt: metrics::Metric,
    k: usize,
    visited_id: FixedBitSet,
    calculation_context: Vec<(Vec<usize>, Vec<usize>, Vec<usize>, Vec<usize>)>, // nn_new_neighbors, nn_old_neighbors, reversed_new_neighbors, reversed_old_neighbors
    rho: f32,
    cost: usize,
    s: usize,
    update_cnt: usize,
}

impl<'a, E: FloatElement, T: IdxType> NNDescentHandler<'a, E, T> {
    fn new(nodes: &'a [Box<Node<E, T>>], mt: metrics::Metric, k: usize, rho: f32) -> Self {
        NNDescentHandler {
            nodes,
            graph: Vec::new(), // TODO: as params
            mt,
            k,
            visited_id: FixedBitSet::with_capacity(nodes.len() * nodes.len()),
            calculation_context: Vec::new(),
            rho,
            cost: 0,
            s: (rho * k as f32) as usize,
            update_cnt: 0,
        }
    }

    fn update(
        &self,
        u1: usize,
        u2: usize,
        my_graph: &[Arc<Mutex<BinaryHeap<Neighbor<E, usize>>>>],
    ) -> bool {
        if u1 == u2 {
            return false;
        }

        self.update_nn_node(u1, u2, my_graph);
        self.update_nn_node(u2, u1, my_graph);
        true
    }

    fn update_nn_node(
        &self,
        me: usize,
        candidate: usize,
        my_graph: &[Arc<Mutex<BinaryHeap<Neighbor<E, usize>>>>],
    ) -> bool {
        let dist = self.nodes[me]
            .metric(&self.nodes[candidate], self.mt)
            .unwrap();
        if dist > my_graph[me].lock().unwrap().peek().unwrap().distance() {
            false
        } else {
            my_graph[me]
                .lock()
                .unwrap()
                .push(Neighbor::new(candidate, dist));
            if my_graph[me].lock().unwrap().len() > self.k {
                my_graph[me].lock().unwrap().pop();
            }
            true
        }
    }

    fn init(&mut self) {
        self.visited_id = FixedBitSet::with_capacity(self.nodes.len() * self.nodes.len());
        self.graph.clear();

        self.graph = (0..self.nodes.len())
            .into_par_iter()
            .map(|_i| {
                let mut v = BinaryHeap::with_capacity(self.k * 2);
                for _j in 0..self.k {
                    v.push(Neighbor::new(self.nodes.len(), E::max_value()));
                }
                Arc::new(Mutex::new(v))
            })
            .collect();

        self.calculation_context = (0..self.nodes.len())
            .into_par_iter()
            .map(|_i| {
                let mut nn_new_neighbors: Vec<usize> = Vec::with_capacity(self.s);
                let nn_old_neighbors: Vec<usize> = Vec::with_capacity(self.s);
                for _j in 0..self.s {
                    let rand_val = rand::thread_rng().gen_range(0..self.nodes.len());
                    nn_new_neighbors.push(rand_val);
                }