// this lib migrate from official lib, but without std dependency;

use core::mem::{swap, ManuallyDrop};
use core::ptr;

pub struct BinaryHeap<T> {
    data: Vec<T>,
}

impl<T: Ord> BinaryHeap<T