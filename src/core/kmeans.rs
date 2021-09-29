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
    mt: metrics::Metric, //compute metrics
}

impl<E: node::FloatElement> Kmeans<E> {
    pub fn new(dimension: usize, n_center: usize, mt: metrics::Metric) -> Kmeans<E> {
        Kmeans {
            _dimension: dimension,
            _n_center: n_center,
            _data_range_begin: 0,
            _data_range_end: dimension,
            mt,
            ..Default::default()
        }
    }

    pub fn centers(&self) -> &Vec<Vec<E>> {
        &self._centers
    }

    pub fn get_distance_from_vec(&self,