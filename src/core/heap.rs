// this lib migrate from official lib, but without std dependency;

use core::mem::{swap, ManuallyDrop};
use core::ptr;

pub struct BinaryHeap<T> {
    data: Vec<T>,
}

impl<T: Ord> BinaryHeap<T> {
    pub fn new() -> BinaryHeap<T> {
        BinaryHeap { data: vec![] }
    }

    pub fn with_capacity(capacity: usize) -> BinaryHeap<T