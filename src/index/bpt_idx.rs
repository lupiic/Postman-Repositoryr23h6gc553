
#![allow(dead_code)]
use crate::core::ann_index;
use crate::core::calc;
use crate::core::metrics;
use crate::core::neighbor;
use crate::core::node;
use crate::core::random;
use crate::index::bpt_params::BPTParams;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fs::File;

use std::io::Write;

// TODO: leaf as a trait with getter setter function
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
struct Leaf<E: node::FloatElement, T: node::IdxType> {
    n_descendants: i32, // tot n_descendants
    children: Vec<i32>, // left and right and if it's a leaf leaf, children would be very large (depend on _K)
    #[serde(skip_serializing, skip_deserializing)]
    node: Box<node::Node<E, T>>,
    tmp_node: Option<node::Node<E, T>>,

    // biz field
    norm: E,
    has_init: bool,
}

impl<E: node::FloatElement, T: node::IdxType> Leaf<E, T> {
    fn new() -> Leaf<E, T> {
        Leaf {
            children: vec![0, 0],
            ..Default::default()
        }
    }

    fn new_with_vectors(_v: &[E]) -> Leaf<E, T> {
        Leaf {
            children: vec![0, 0],
            node: Box::new(node::Node::new(_v)),
            ..Default::default()
        }
    }

    fn new_with_item(_v: &node::Node<E, T>) -> Leaf<E, T> {
        Leaf {
            children: vec![0, 0],
            node: Box::new(_v.clone()),
            ..Default::default()
        }
    }

    fn is_empty(&self) -> bool {
        self.has_init
    }

    fn init(&mut self) {
        self.children = vec![0, 0];
    }

    fn clone_node(&self) -> node::Node<E, T> {
        *self.node.clone()
    }

    fn normalize(&mut self) {
        let norm = calc::get_norm(self.node.vectors()).unwrap();
        if norm > E::float_zero() {
            for i in 0..self.node.len() {
                self.node.mut_vectors()[i] /= norm;
            }
        }
    }

    fn copy(dst: &mut Leaf<E, T>, src: &Leaf<E, T>) {
        dst.n_descendants = src.n_descendants;
        dst.children = src.children.clone();
        dst.node = src.node.clone();
        dst.norm = src.norm;
    }

    pub fn get_literal(&self) -> String {
        format!(
            "{{ \"n_descendants\": {:?}, \"children\": {:?}, \"has_init\": {:?} }}, \"node\": {:?},",
            self.n_descendants, self.children, self.has_init, *self.node
        )
    }

    // replace distance copy_leaf
    fn copy_leaf(src: &Leaf<E, T>) -> Leaf<E, T> {
        Leaf {
            n_descendants: src.n_descendants,
            node: src.node.clone(),
            children: src.children.clone(),
            ..Default::default()
        }
    }
}

fn two_means<E: node::FloatElement, T: node::IdxType>(
    leaves: &[Leaf<E, T>],
    mt: metrics::Metric,
) -> Result<(Leaf<E, T>, Leaf<E, T>), &'static str> {
    const ITERATION_STEPS: usize = 200;
    if leaves.len() < 2 {
        return Err("empty leaves");
    }

    let count = leaves.len();

    let i = random::index(count);
    let mut j = random::index(count - 1);
    // make sure j not equal to i;
    if j >= i {
        j += 1;
    }

    let mut first = Leaf::copy_leaf(&leaves[i]);
    let mut second = Leaf::copy_leaf(&leaves[j]);

    if mt == metrics::Metric::CosineSimilarity {
        first.normalize();
        second.normalize();
    }
    // TODO: dot normalize

    let one = E::float_one();
    let zero = E::float_zero();

    let mut ic: E = one;
    let mut jc: E = one;

    // produce two mean point.
    for _z in 0..ITERATION_STEPS {
        let rand_k = random::index(count);
        let di =
            ic * metrics::metric(first.node.vectors(), leaves[rand_k].node.vectors(), mt).unwrap();
        let dj =
            jc * metrics::metric(second.node.vectors(), leaves[rand_k].node.vectors(), mt).unwrap();

        //
        let mut norm = one;
        if mt == metrics::Metric::CosineSimilarity {
            norm = calc::get_norm(leaves[rand_k].node.vectors()).unwrap();
            match norm.partial_cmp(&zero) {
                Some(Ordering::Equal) | Some(Ordering::Less) => continue,
                _ => {}
            };
        }

        // make p more closer to k in space.
        if di < dj {
            for l in 0..first.node.len() {
                first.node.mut_vectors()[l] = (first.node.vectors()[l] * ic
                    + leaves[rand_k].node.vectors()[l] / norm)
                    / (ic + one);
            }
            ic += one;
        } else if dj < di {
            for l in 0..second.node.len() {
                second.node.mut_vectors()[l] = (second.node.vectors()[l] * jc
                    + leaves[rand_k].node.vectors()[l] / norm)
                    / (jc + one);
            }
            jc += one;
        }
    }
    Ok((first, second))
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct BPTIndex<E: node::FloatElement, T: node::IdxType> {
    _dimension: usize,    // dimension
    _tot_items_cnt: i32, // add items count, means the physically the item count, _tot_items_cnt == leaves.size()
    _tot_leaves_cnt: i32, // leaves count, whole tree leaves count
    // _leaves_size: i32, // in source code, this means the memory which has been allocated, and we can use leaf's size to get data
    _roots: Vec<i32>,     // dummy root's children
    _leaf_max_items: i32, // max number of n_descendants to fit into leaf
    _built: bool,
    leaves: Vec<Leaf<E, T>>,
    mt: metrics::Metric,
    _tree_num: i32,
    _candidate_size: i32,
}

impl<E: node::FloatElement, T: node::IdxType> BPTIndex<E, T> {
    pub fn new(dimension: usize, params: &BPTParams) -> BPTIndex<E, T> {
        BPTIndex {
            _built: false,
            _dimension: dimension,
            _leaf_max_items: ((dimension / 2) as i32) + 2,
            _tree_num: params.tree_num,
            _candidate_size: params.candidate_size,
            leaves: vec![Leaf::new()], // the id count should start from 1, use a node as placeholder
            ..Default::default()
        }
    }

    fn _add_item(&mut self, w: &node::Node<E, T>) -> Result<(), &'static str> {
        // TODO: remove
        if w.len() != self._dimension {
            return Err("dimension is different");
        }

        let mut nn = Leaf::new_with_item(w);

        nn.children[0] = 0; // TODO: as const value
        nn.children[1] = 0;
        nn.n_descendants = 1; // only the leaf itself, so the n_descendants include it self

        // no update method
        self._tot_items_cnt += 1;

        self.leaves.push(nn);

        Ok(())
    }

    fn build(&mut self, mt: metrics::Metric) -> Result<(), &'static str> {
        if self._built {
            return Err("has built");
        }

        self.mt = mt;
        self._tot_leaves_cnt = self._tot_items_cnt; // init with build.
        self._build(self._tree_num, self.mt);
        self._built = true;
        Ok(())
    }

    fn clear(&mut self) {
        self._roots.clear();
        self._tot_leaves_cnt = self._tot_items_cnt;
        self._built = false;
    }
    fn get_distance(&self, i: i32, j: i32) -> E {
        let ni = self.get_leaf(i).unwrap();
        let nj = self.get_leaf(j).unwrap();
        return metrics::metric(ni.node.vectors(), nj.node.vectors(), self.mt).unwrap();
    }

    fn get_tot_items_cnt(&self) -> i32 {
        self._tot_items_cnt
    }
    fn get_n_tree(&self) -> i32 {
        self._roots.len() as i32
    }

    fn get_dimension(&self) -> usize {
        self._dimension
    }

    fn get_k(&self) -> i32 {
        self._leaf_max_items
    }

    fn get_leaf_mut(&mut self, i: i32) -> &mut Leaf<E, T> {
        if self.leaves.len() <= i as usize {
            self.extent_leaves(i as usize);
        }
        &mut self.leaves[i as usize]
    }

    fn extent_leaf(&mut self) -> &mut Leaf<E, T> {
        let i = self.leaves.len();
        self.extent_leaves(self.leaves.len());
        if self.leaves[i].is_empty() {
            self.leaves[i].init();
        }
        &mut self.leaves[i]
    }

    fn get_leaf(&self, i: i32) -> Option<&Leaf<E, T>> {
        if self.leaves.len() < i as usize {
            return None;
        }
        if self.leaves[i as usize].is_empty() {
            return None;
        }
        Some(&self.leaves[i as usize])
    }

    fn extent_leaves(&mut self, i: usize) {
        let diff = i - self.leaves.len() + 1;
        if diff > 0 {
            for _i in 0..diff {
                self.leaves.push(Leaf::new());
            }
        }
    }

    // q => tree count
    // TODO: build failed
    fn _build(&mut self, tree_num: i32, mt: metrics::Metric) {
        let mut this_root: Vec<i32> = Vec::new();

        loop {
            if tree_num == -1 {
                if self._tot_leaves_cnt >= 2 * self._tot_items_cnt {
                    break;
                }
            } else if this_root.len() >= (tree_num as usize) {
                break;
            }

            let mut indices: Vec<i32> = Vec::new();
            for i in 1..self._tot_items_cnt {
                let leaf = self.get_leaf(i).unwrap();
                if leaf.n_descendants >= 1 {
                    indices.push(i as i32);
                }
            }

            let tree = self.make_tree(&indices, true, mt).unwrap();
            this_root.push(tree);
        }

        // thread lock
        self._roots.extend_from_slice(&this_root);
    }

    fn make_tree(
        &mut self,
        indices: &[i32],
        is_root: bool,
        mt: metrics::Metric,
    ) -> Result<i32, &'static str> {
        if indices.is_empty() {
            return Err("empty indices");
        }
        if indices.len() == 1 && !is_root {
            return Ok(indices[0]);
        }

        // the batch is a leaf cluster, make a parent node
        if (indices.len() as i32) <= self._leaf_max_items
            && (!is_root || self._tot_items_cnt <= self._leaf_max_items || indices.len() == 1)