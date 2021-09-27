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
    _cente