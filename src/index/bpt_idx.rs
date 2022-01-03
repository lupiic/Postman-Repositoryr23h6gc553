
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