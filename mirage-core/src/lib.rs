#![feature(immovable_types)]
use std::marker::Move;

extern crate mirage_async;
use mirage_async::{Async, Await};

pub struct Core;

impl Core {
    pub fn new() -> Core {
        Core
    }

    pub fn run<T, A: Async<T> + ?Move>(&mut self, mut a: A) -> T {
        loop {
            if let Await::Done(r) = a.poll() {
                return r;
            }
        }
    }
}
