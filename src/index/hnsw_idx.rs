#![allow(dead_code)]
use crate::core::ann_index;
use crate::core::metrics;
use crate::core::neighbor::Neighbor;
use crate::core::node;
use crate::index::hnsw_params::HNSWParams;
use crate::into_iter;
use fixedbitset::FixedBitSet;
use rand::prelude::*;
#[cfg(not(feature = "no_thread"))]
use rayon::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

use std::sync::RwLock;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct HNSWIndex<E: node::FloatElement, T: node::IdxType> {
    _dimension: usize, // dimension
    _n_items: usize,   // next item count
    _n_constructed_items: usize,
    _max_item: usize,
    _n_neighbor: usize,  // neighbor num except level 0
    _n_neighbor0: usize, // neight num of level 0
    _max_level: usize,   //max level
    _cur_level: usize,   //current level
    #[serde(skip_serializing, skip_deserializing)]
    _id2neighbor: Vec<Vec<RwLock<Vec<usize>>>>, //neight_id from level 1 to level _max_level
    #[serde(skip_serializing, skip_deserializing)]
    _id2neighbor0: Vec<RwLock<Vec<usize>>>, //neigh_id at level 0
    #[serde(skip_serializing, skip_deserializing)]
    _nodes: Vec<Box<node::Node<E, T>>>, // data saver
    #[serde(skip_serializing, skip_deserializing)]
    _item2id: HashMap<T, usize>, //item_id to id in Hnsw
    _root_id: usize,     //root of hnsw
    _id2level: Vec<usize>,
    _has_removed: bool,
    _ef_build: usize,  // num of max candidates when building
    _ef_search: usize, // num of max candidates when searching
    #[serde(skip_serializing, skip_deserializing)]
    _delete_ids: HashSet<usize>, //save deleted ids
    mt: metrics::Metric, //compute metrics

    // use for serde
    _id2neighbor_tmp: Vec<Vec<Vec<usize>>>,
    _id2neighbor0_tmp: Vec<Vec<usize>>,
    _nodes_tmp: Vec<node::Node<E, T>>,
    _item2id_tmp: Vec<(T, usize)>,
    _delete_ids_tmp: Vec<usize>,
}

impl<E: node::FloatElement, T: node::IdxType> HNSWIndex<E, T> {
    pub fn new(dimension: usize, params: &HNSWParams<E>) -> HNSWIndex<E, T> {
        HNSWIndex {
            _dimension: dimension,
            _n_items: 0,
            _n_constructed_items: 0,
            _max_item: params.max_item,
            _n_neighbor: params.n_neighbor,
            _n_neighbor0: params.n_neighbor0,
            _max_level: params.max_level,
            _cur_level: 0,
            _root_id: 0,
            _has_removed: params.has_deletion,
            _ef_build: params.ef_build,
            _ef_search: params.ef_search,
            mt: metrics::Metric::Unknown,
            ..Default::default()
        }
    }

    fn get_random_level(&self) -> usize {
        let mut rng = rand::thread_rng();
        let mut ret = 0;
        while ret < self._max_level {
            if rng.gen_range(0.0..1.0) > 0.5 {
                ret += 1;
            } else {
                break;
            }
        }
        ret
    }
    //input top_candidate as max top heap
    //return min top heap in top_candidates, delete part candidate
    fn get_neighbors_by_heuristic2(
        &self,
        sorted_list: &[Neighbor<E, usize>],
        ret_size: usize,
    ) -> Vec<Neighbor<E, usize>> {
        let sorted_list_len = sorted_list.len();
        let mut return_list: Vec<Neighbor<E, usize>> = Vec::with_capacity(sorted_list_len);

        for iter in sorted_list.iter() {
            if return_list.len() >= ret_size {
                break;
            }

            let idx = iter.idx();
            let distance = iter._distance;
            if sorted_list_len < ret_size {
                return_list.push(Neighbor::new(idx, distance));
                continue;
            }

            let mut good = true;

            for ret_neighbor in return_list.iter() {
                let cur2ret_dis = self.get_distance_from_id(idx, ret_neighbor.idx());
                if cur2ret_dis < distance {
                    good = false;
                    break;
                }
            }

            if good {
                return_list.push(Neighbor::new(idx, distance));
            }
        }

        return_list // from small to large
    }

    fn get_neighbor(&self, id: usize, level: usize) -> &RwLock<Vec<usize>> {
        if level == 0 {
            return &self._id2neighbor0[id];
        }
        &self._id2neighbor[id][level - 1]
    }

    #[allow(dead_code)]
    fn get_level(&self, id: usize) -> usize {
        self._id2level[id]
    }

    fn connect_neighbor(
        &self,
        cur_id: usize,
        sorted_candidates: &[Neighbor<E, usize>],
        level: usize,
        is_update: bool,
    ) -> Result<usize, &'static str> {
        let n_neigh = if level == 0 {
            self._n_neighbor0
        } else {
            self._n_neighbor
        };
        let selected_neighbors = self.get_neighbors_by_heuristic2(sorted_candidates, n_neigh);
        if selected_neighbors.len() > n_neigh {
            return Err("Should be not be more than M_ candidates returned by the heuristic");
        }
        if selected_neighbors.is_empty() {
            return Err("top candidate is empty, impossible!");
        }

        let next_closest_entry_point = selected_neighbors[0].idx();

        {
            let mut cur_neigh = self.get_neighbor(cur_id, level).write().unwrap();
            cur_neigh.clear();
            selected_neighbors.iter().for_each(|selected_neighbor| {
                cur_neigh.push(selected_neighbor.idx());
            });
        }

        for selected_neighbor in selected_neighbors.iter() {
            let mut neighbor_of_selected_neighbors = self
                .get_neighbor(selected_neighbor.idx(), level)
                .write()
                .unwrap();
            if neighbor_of_selected_neighbors.len() > n_neigh {
                return Err("Bad Value of neighbor_of_selected_neighbors");
            }
            if selected_neighbor.idx() == cur_id {
                return Err("Trying to connect an ele