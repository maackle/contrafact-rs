use std::marker::PhantomData;

use crate::*;

/// Create a fact from a bare function which specifies the mutation
pub fn fun<'a, T, S, F>(state: S, f: F) -> FunFact<'a, T, S, F>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
    F: Clone + Send + Sync + FnMut(&mut Generator, &mut S, T) -> Mutation<T>,
{
    FunFact {
        state,
        fun: f,
        _phantom: PhantomData,
    }
}

#[derive(Clone)]
pub struct FunFact<'a, T, S, F>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
    F: Clone + Send + Sync + FnMut(&mut Generator, &mut S, T) -> Mutation<T>,
{
    state: S,
    fun: F,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T, S, F> Fact<'a, T> for FunFact<'a, T, S, F>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
    F: Clone + Send + Sync + FnMut(&mut Generator, &mut S, T) -> Mutation<T>,
{
    fn mutate(&mut self, g: &mut Generator<'a>, obj: T) -> Mutation<T> {
        (self.fun)(g, &mut self.state, obj)
    }
}
