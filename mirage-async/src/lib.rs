#![feature(generators)]
#![feature(generator_trait)]
#![feature(immovable_types)]

#[macro_export]
mod macros;

use std::ops::{Generator, GeneratorState};
use std::marker::Move;

pub trait Async<R>: ?Move {
    fn poll(&mut self) -> Await<R>;
}

pub enum Await<T> {
    Done(T),
    NotReady,
}

pub struct AsAsync<T: ?Move>(pub T);

impl<T: Generator<Return = R, Yield = ()> + ?Move, R> Async<R> for AsAsync<T> {
    fn poll(&mut self) -> Await<R> {
        self.0.resume().into()
    }
}

impl<R> From<GeneratorState<(), R>> for Await<R> {
    fn from(f: GeneratorState<(), R>) -> Await<R> {
        match f {
            GeneratorState::Yielded(()) => Await::NotReady,
            GeneratorState::Complete(r) => Await::Done(r),
        }
    }
}
