
#![allow(dead_code)]
use crate::core::ann_index;
use crate::core::kmeans;
use crate::core::metrics;
use crate::core::neighbor::Neighbor;
use crate::core::node;
use crate::index::pq_params::IVFPQParams;
use crate::index::pq_params::PQParams;
use crate::vec_iter_mut;
#[cfg(not(feature = "no_thread"))]
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use std::collections::BinaryHeap;

use serde::{Deserialize, Serialize};

use std::fs::File;

use std::io::Write;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct PQIndex<E: node::FloatElement, T: node::IdxType> {
    _dimension: usize,                 //dimension of data
    _n_sub: usize,                     //num of subdata
    _sub_dimension: usize,             //dimension of subdata
    _dimension_range: Vec<Vec<usize>>, //dimension preset
    _sub_bits: usize,                  // size of subdata code
    _sub_bytes: usize,                 //code save as byte: (_sub_bit + 7)//8
    _n_sub_center: usize,              //num of centers per subdata code
    //n_center_per_sub = 1 << sub_bits
    _code_bytes: usize,         // byte of code
    _train_epoch: usize,        // training epoch
    _centers: Vec<Vec<Vec<E>>>, // size to be _n_sub * _n_sub_center * _sub_dimension
    _is_trained: bool,
    _has_residual: bool,
    _residual: Vec<E>,

    _n_items: usize,
    _max_item: usize,
    _nodes: Vec<Box<node::Node<E, T>>>,
    _assigned_center: Vec<Vec<usize>>,
    mt: metrics::Metric, //compute metrics
    // _item2id: HashMap<i32, usize>,
    _nodes_tmp: Vec<node::Node<E, T>>,
}

impl<E: node::FloatElement, T: node::IdxType> PQIndex<E, T> {
    pub fn new(dimension: usize, params: &PQParams<E>) -> PQIndex<E, T> {
        let n_sub = params.n_sub;
        let sub_bits = params.sub_bits;
        let train_epoch = params.train_epoch;
        let sub_dimension = dimension / n_sub;

        let sub_bytes = (sub_bits + 7) / 8;
        assert!(sub_bits <= 32);
        let n_center_per_sub = (1 << sub_bits) as usize;
        let code_bytes = sub_bytes * n_sub;
        let mut new_pq = PQIndex::<E, T> {
            _dimension: dimension,
            _n_sub: n_sub,
            _sub_dimension: sub_dimension,
            _sub_bits: sub_bits,
            _sub_bytes: sub_bytes,
            _n_sub_center: n_center_per_sub,
            _code_bytes: code_bytes,
            _train_epoch: train_epoch,
            _is_trained: false,
            _n_items: 0,
            _max_item: 100000,
            _has_residual: false,
            mt: metrics::Metric::Euclidean,
            ..Default::default()
        };

        for i in 0..n_sub {
            let begin;
            let end;
            if i < dimension % sub_dimension {
                begin = i * (sub_dimension + 1);
                end = (i + 1) * (sub_dimension + 1);
            } else {
                begin = (dimension % sub_dimension) * (sub_dimension + 1)
                    + (i - dimension % sub_dimension) * sub_dimension;
                end = (dimension % sub_dimension) * (sub_dimension + 1)
                    + (i + 1 - dimension % sub_dimension) * sub_dimension;
            };
            new_pq._dimension_range.push(vec![begin, end]);
        }
        new_pq
    }

    fn init_item(&mut self, data: &node::Node<E, T>) -> usize {
        let cur_id = self._n_items;
        // self._item2id.insert(item, cur_id);
        self._nodes.push(Box::new(data.clone()));
        self._n_items += 1;
        cur_id
    }

    fn add_item(&mut self, data: &node::Node<E, T>) -> Result<usize, &'static str> {
        if data.len() != self._dimension {
            return Err("dimension is different");
        }
        // if self._item2id.contains_key(&item) {
        //     //to_do update point
        //     return Ok(self._item2id[&item]);
        // }

        if self._n_items > self._max_item {
            return Err("The number of elements exceeds the specified limit");
        }

        let insert_id = self.init_item(data);
        Ok(insert_id)
    }

    fn set_residual(&mut self, residual: Vec<E>) {
        self._has_residual = true;
        self._residual = residual;
    }

    fn train_center(&mut self) {
        let n_item = self._n_items;
        let n_sub = self._n_sub;
        (0..n_sub).for_each(|i| {
            let _dimension = self._sub_dimension;
            let n_center = self._n_sub_center;
            let n_epoch = self._train_epoch;
            let begin = self._dimension_range[i][0];
            let end = self._dimension_range[i][1];
            let mut data_vec: Vec<Vec<E>> = Vec::new();
            for node in self._nodes.iter() {
                data_vec.push(node.vectors().to_vec());
            }

            let mut cluster = kmeans::Kmeans::<E>::new(end - begin, n_center, self.mt);
            cluster.set_range(begin, end);
            if self._has_residual {
                cluster.set_residual(self._residual.to_vec());
            }

            cluster.train(n_item, &data_vec, n_epoch);
            let mut assigned_center: Vec<usize> = Vec::new();
            cluster.search_data(n_item, &data_vec, &mut assigned_center);
            self._centers.push(cluster.centers().to_vec());
            self._assigned_center.push(assigned_center);
        });
        self._is_trained = true;
    }

    fn get_distance_from_vec_range(
        &self,
        x: &node::Node<E, T>,
        y: &[E],
        begin: usize,
        end: usize,
    ) -> E {
        let mut z = x.vectors()[begin..end].to_vec();
        if self._has_residual {
            (0..end - begin).for_each(|i| z[i] -= self._residual[i + begin]);
        }
        return metrics::metric(&z, y, self.mt).unwrap();
    }

    fn search_knn_adc(
        &self,
        search_data: &node::Node<E, T>,
        k: usize,
    ) -> Result<BinaryHeap<Neighbor<E, usize>>, &'static str> {
        let mut dis2centers: Vec<E> = Vec::new();
        dis2centers.resize(self._n_sub * self._n_sub_center, E::from_f32(0.0).unwrap());
        vec_iter_mut!(dis2centers, ctr);
        ctr.enumerate().for_each(|(idx, x)| {
            let i = idx / self._n_sub_center;
            let j = idx % self._n_sub_center;
            let begin = self._dimension_range[i][0];
            let end = self._dimension_range[i][1];
            *x = self.get_distance_from_vec_range(search_data, &self._centers[i][j], begin, end);