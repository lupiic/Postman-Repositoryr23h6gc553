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

impl<E: node::FloatElement> PQParams<E> {
    pub fn n_sub(mut self, new_n_sub: usize) -> Self {
        self.n_sub = new_n_sub;
        self
    }

    pub fn sub_bits(mut self, new_sub_bits: usize) -> Self {
        self.sub_bits = new_sub_bits;
        self
    }

    pub fn train_epoch(mut self, new_train_epoch: usize) -> Self {
        self.train_epoch = 