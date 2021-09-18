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

    pub fn with_capacity(capacity: usize) -> BinaryHeap<T> {
        BinaryHeap {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop().map(|mut item| {
            if !self.is_empty() {
                swap(&mut item, &mut self.data[0]);
                self.sift_down_to_bottom(0);
            }
            item
        })
    }

    pub fn push(&mut self, item: T) {
        let old_len = self.len();
        self.data.push(item);
        self.sift_up(0, old_len);
    }

    pub fn into_sorted_vec(mut self) -> Vec<T> {
        let mut end = self.len();
        while end > 1 {
            end -= 1;
            // SAFETY: `end` goes from `self.len() - 1` to 1 (both included),
            //  so it's always a valid index to access.
            //  It is safe to access index 0 (i.e. `ptr`), because
            //  1 <= end < self.len(), which means self.len() >= 2.
            unsafe {
                let ptr = self.data.as_mut_ptr();
                ptr::swap(ptr, ptr.add(end));
            }
            self.sift_down_range(0, end);
        }
        self.into_vec()
    }

    fn sift_up(&mut self, start: usize, pos: usize) -> usize {
        unsafe {
            // Take out the value at `pos` and create a hole.
            let mut hole = Hole::new(&mut self.data, pos);

            while hole.pos() > start {
                let parent = (hole.pos() - 1) / 2;
                if hole.element() <= hole.get(parent) {
                    break;
                }
                hole.move_to(parent);
            }
            hole.pos()
        }
    }

    fn sift_down_range(&mut self, pos: usize, end: usize) {
        unsafe {
            let mut hole = Hole::new(&mut self.data, pos);
            let mut child = 2 * pos + 1;
            while child < end - 1 {
                // compare with the greater of the two children
                child += (hole.get(child) <= hole.get(child + 1)) as usize;
                // if we are already in order, stop.
                if hole.element() >= hole.get(child) {
                    return;
                }
                hole.move_to(child);
                child = 2 * hole.pos() + 1;
            }
            if child == end - 1 && hole.element() < hole.get(child) {
                hole.move_to(child);
            }
        }
    }

    fn sift_down(&mut self, pos: usize) {
        let len = self.len();
        self.sift_down_range(pos, len);
    }

    fn sift_down_to_bottom(&mut self, mut pos: usize) {
        let end = self.len();
        let start = pos