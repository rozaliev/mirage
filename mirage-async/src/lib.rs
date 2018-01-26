#![feature(generators)]
#![feature(generator_trait)]
#![feature(immovable_types)]

mod macros;

use std::ops::{Generator, GeneratorState};

pub trait Async<R> {
    unsafe fn poll(&mut self) -> Await<R>;
}

pub enum Await<T> {
    Done(T),
    NotReady,
}

pub struct AsAsync<T>(pub T);

impl<T: Generator<Return = R, Yield = ()>, R> Async<R> for AsAsync<T> {
    unsafe fn poll(&mut self) -> Await<R> {
        self.0.resume().into()
    }
}
impl<T: Generator<Return = R, Yield = ()>, R> Async<R> for Box<T> {
    unsafe fn poll(&mut self) -> Await<R> {
        (*self).resume().into()
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
