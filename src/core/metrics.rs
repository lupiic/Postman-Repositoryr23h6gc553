
extern crate num;
use crate::core::{calc::dot, node::FloatElement};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Metric {
    Unknown,
    Manhattan,
    DotProduct,
    Euclidean,
    CosineSimilarity,
    Angular,
}

impl Default for Metric {
    fn default() -> Self {
        Metric::Unknown
    }
}

// TODO: make these func private
pub fn metric<T>(vec1: &[T], vec2: &[T], mt: Metric) -> Result<T, &'static str>