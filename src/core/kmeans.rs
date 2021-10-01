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

    pub fn get_distance_from_vec(&self, x: &[E], y: &[E]) -> E {
        let mut z = x[self._data_range_begin..self._data_range_end].to_vec();
        if self._has_residual {
            (0..self._data_range_end - self._data_range_begin)
                .for_each(|i| z[i] -= self._residual[i + self._data_range_begin]);
        }
        return metric(&z, y, self.mt).unwrap();
    }

    pub fn set_residual(&mut self, residual: Vec<E>) {
        self._has_residual = true;
        self._residual = residual;
    }

    pub fn init_center(&mut self, batch_size: usize, batch_data: &[Vec<E>]) {
        let dimension = self._dimension;
        let n_center = self._n_center;
        let begin = self._data_range_begin;
        let mut mean_center: Vec<E> = vec![E::from_f32(0.0).unwrap(); dimension];

        (0..batch_size).for_each(|i| {
            let cur_data = &batch_data[i];
            (0..dimension).for_each(|j| {
                if self._has_residual {
                    mean_