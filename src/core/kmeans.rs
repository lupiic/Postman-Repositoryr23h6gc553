#![allow(dead_code)]
use crate::core::{metrics, node};
use metrics::metric;
use rand::prelude::*;
use rayon::prelude::*;
use std::sync::Mutex;

#[derive(Default, Debug)]
pub struct Kmeans<E: node::FloatElement> {
    _dimension: usize,
    _n_center: usize,
    _centers: Vec<Vec<E>>,
    _data_range_begin: usize,
    _data_range_end: usize,
    _has_residual: bool,
    _residual: Vec<E>,
   