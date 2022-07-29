#![allow(dead_code)]

use crate::core::node;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PQParams<E: node::FloatElement> {
    pub n_sub: usize,
    pub sub_bits: usize,
    pub train_epoch: usize,
    pub e_type: E,
}

impl<E: node::FloatElemen