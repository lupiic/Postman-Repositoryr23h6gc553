extern crate num;
use crate::core::node;
use core::cmp::Ordering;

/// internal temp structure
///
#[derive(Default, Clone, PartialEq, Debug)]
pub struct Neighbor<E: node::FloatElement, T: node::IdxType> {
    pub 