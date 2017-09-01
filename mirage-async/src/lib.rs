#![feature(generators)]
#![feature(generator_trait)]

#[macro_export]
mod macros;

use std::ops::{Generator, GeneratorState};

pub trait Async<R> {
    fn poll(&mut self) -> Await<R>;
}

pub enum Await<T> {
    Done(T),
    NotReady,
}


impl<T, R> Async<R> for T
where
    T: Generator<Return = R, Yield = ()>,
{
    fn poll(&mut self) -> Await<R> {
        self.resume().into()
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
