#![allow(dead_code)]


use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BPTParams {
    pub tree_num: i32,
    pub candidate_size: i32,
}

impl BPTParams {
    pub fn tree_num(mut self, new_tree_num: i32) -> Self {
        self.tree_num = new_tree_num;
        self
    }
    pub fn candidate_size(mut self, new_candidate_size: i32) -> Self {
        self.candidate_