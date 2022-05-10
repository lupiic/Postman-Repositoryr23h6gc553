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
                return Err("Trying to connect an element to itself");
            }

            let mut is_cur_id_present = false;

            if is_update {
                for iter in neighbor_of_selected_neighbors.iter() {
                    if *iter == cur_id {
                        is_cur_id_present = true;
                        break;
                    }
                }
            }

            if !is_cur_id_present {
                if neighbor_of_selected_neighbors.len() < n_neigh {
                    neighbor_of_selected_neighbors.push(cur_id);
                } else {
                    let d_max = self.get_distance_from_id(cur_id, selected_neighbor.idx());

                    let mut candidates: BinaryHeap<Neighbor<E, usize>> = BinaryHeap::new();
                    candidates.push(Neighbor::new(cur_id, d_max));
                    for iter in neighbor_of_selected_neighbors.iter() {
                        let neighbor_id = *iter;
                        let d_neigh =
                            self.get_distance_from_id(neighbor_id, selected_neighbor.idx());
                        candidates.push(Neighbor::new(neighbor_id, d_neigh));
                    }
                    let return_list =
                        self.get_neighbors_by_heuristic2(&candidates.into_sorted_vec(), n_neigh);

                    neighbor_of_selected_neighbors.clear();
                    for neighbor_in_list in return_list {
                        neighbor_of_selected_neighbors.push(neighbor_in_list.idx());
                    }
                }
            }
        }

        Ok(next_closest_entry_point)
    }

    #[allow(dead_code)]
    fn delete_id(&mut self, id: usize) -> Result<(), &'static str> {
        if id > self._n_constructed_items {
            return Err("Invalid delete id");
        }
        if self.is_deleted(id) {
            return Err("id has deleted");
        }
        self._delete_ids.insert(id);
        Ok(())
    }

    fn is_deleted(&self, id: usize) -> bool {
        self._has_removed && self._delete_ids.contains(&id)
    }

    fn get_data(&self, id: usize) -> &node::Node<E, T> {
        &self._nodes[id]
    }

    fn get_distance_from_vec(&self, x: &node::Node<E, T>, y: &node::Node<E, T>) -> E {
        return metrics::metric(x.vectors(), y.vectors(), self.mt).unwrap();
    }

    fn get_distance_from_id(&self, x: usize, y: usize) -> E {
        return metrics::metric(
            self.get_data(x).vectors(),
            self.get_data(y).vectors(),
            self.mt,
        )
        .unwrap();
    }

    fn search_layer_with_candidate(
        &self,
        search_data: &node::Node<E, T>,
        sorted_candidates: &[Neighbor<E, usize>],
        visited_id: &mut FixedBitSet,
        level: usize,
        ef: usize,
        has_deletion: bool,
    ) -> BinaryHeap<Neighbor<E, usize>> {
        let mut candidates: BinaryHeap<Neighbor<E, usize>> = BinaryHeap::new();
        let mut top_candidates: BinaryHeap<Neighbor<E, usize>> = BinaryHeap::new();
        for neighbor in sorted_candidates.iter() {
            let root = neighbor.idx();
            if !has_deletion || !self.is_deleted(root) {
                let dist = self.get_distance_from_vec(self.get_data(root), search_data);
                top_candidates.push(Neighbor::new(root, dist));
                candidates.push(Neighbor::new(root, -dist));
            } else {
                candidates.push(Neighbor::new(root, -E::max_value()))
            }
            visited_id.insert(root);
        }
        let mut lower_bound = if top_candidates.is_empty() {
            E::max_value() //max dist in top_candidates
        } else {
            top_candidates.peek().unwrap()._distance
        };

        while !candidates.is_empty() {
            let cur_neigh = candidates.peek().unwrap();
            let cur_dist = -cur_neigh._distance;
            let cur_id = cur_neigh.idx();
            candidates.pop();
            if cur_dist > lower_bound {
                break;
            }
            let cur_neighbors = self.get_neighbor(cur_id, level).read().unwrap();
            cur_neighbors.iter().for_each(|neigh| {
                if visited_id.contains(*neigh) {
                    return;
                }
                visited_id.insert(*neigh);
                let dist = self.get_distance_from_vec(self.get_data(*neigh), search_data);
                if top_candidates.len() < ef || dist < lower_bound {
                    candidates.push(Neighbor::new(*neigh, -dist));

                    if !self.is_deleted(*neigh) {
                        top_candidates.push(Neighbor::new(*neigh, dist))
                    }

                    if top_candidates.len() > ef {
                        top_candidates.pop();
                    }

                    if !top_candidates.is_empty() {
                        lower_bound = top_candidates.peek().unwrap()._distance;
                    }
                }
            });
        }

        top_candidates
    }
    //find ef nearist nodes to search data from root at level
    fn search_layer(
        &self,
        root: usize,
        search_data: &node::Node<E, T>,
        level: usize,
        ef: usize,
        has_deletion: bool,
    ) -> BinaryHeap<Neighbor<E, usize>> {
        let mut visited_id = FixedBitSet::with_capacity(self._nodes.len());
        let mut top_candidates: BinaryHeap<Neighbor<E, usize>> = BinaryHeap::new();
        let mut candidates: BinaryHeap<Neighbor<E, usize>> = BinaryHeap::new();
        let mut lower_bound: E;

        if !has_deletion || !self.is_deleted(root) {
            let dist = self.get_distance_from_vec(self.get_data(root), search_data);
            top_candidates.push(Neighbor::new(root, dist));
            candidates.push(Neighbor::new(root, -dist));
            lower_bound = dist;
        } else {
            lower_bound = E::max_value(); //max dist in top_candidates
            candidates.push(Neighbor::new(root, -lower_bound))
        }
        visited_id.insert(root);

        while !candidates.is_empty() {
            let cur_neigh = candidates.peek().unwrap();
            let cur_dist = -cur_neigh._distance;
            let cur_id = cur_neigh.idx();
            candidates.pop();
            if cur_dist > lower_bound {
                break;
            }
            let cur_neighbors = self.get_neighbor(cur_id, level).read().unwrap();
            cur_neighbors.iter().for_each(|neigh| {
                if visited_id.contains(*neigh) {
                    return;
                }
                visited_id.insert(*neigh);
                let dist = self.get_distance_from_vec(self.get_data(*neigh), search_data);
                if top_candidates.len() < ef || dist < lower_bound {
                    candidates.push(Neighbor::new(*neigh, -dist));

                    if !self.is_deleted(*neigh) {
                        top_candidates.push(Neighbor::new(*neigh, dist))
                    }

                    if top_candidates.len() > ef {
                        top_candidates.pop();
                    }

                    if !top_candidates.is_empty() {
                        lower_bound = top_candidates.peek().unwrap()._distance;
                    }
                }
            });
        }

        top_candidates
    }

    // fn search_layer_default(
    //     &self,
    //     root: usize,
    //     search_data: &node::Node<E, T>,
    //     level: usize,
    // ) -> BinaryHeap<Neighbor<E, usize>> {
    //     return self.search_layer(root, search_data, level, self._ef_build, false);
    // }

    fn search_knn(
        &self,
        search_data: &node::Node<E, T>,
        k: usize,
    ) -> Result<BinaryHeap<Neighbor<E, usize>>, &'static str> {
        let mut top_candidate: BinaryHeap<Neighbor<E, usize>> = BinaryHeap::new();
        if self._n_constructed_items == 0 {
            return Ok(top_candidate);
        }
        let mut cur_id = self._root_id;
        let mut cur_dist = self.get_distance_from_vec(self.get_data(cur_id), search_data);
        let mut cur_level = self._cur_level;
        loop {
            let mut changed = true;
            while changed {
                changed = false;
                let cur_neighs = self
                    .get_neighbor(cur_id, cur_level as usize)
                    .read()
                    .unwrap();
                for neigh in cur_neighs.iter() {
                    if *neigh > self._max_item {
                        return Err("cand error");
                    }
 