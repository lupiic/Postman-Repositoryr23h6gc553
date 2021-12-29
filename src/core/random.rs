use rand::prelude::*;

pub trait Random<T> {
    fn kiss() -> T;
    fn flip() -> bool;
    fn index(n: usize) -> usize;
}

// TODO: u