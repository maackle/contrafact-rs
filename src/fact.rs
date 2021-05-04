use std::marker::PhantomData;

use crate::{
    constraint::{Bounds, ConstraintBox},
    Constraint,
};
use arbitrary::{Arbitrary, Unstructured};

pub trait Fact<T> {
    fn constraint(&mut self) -> ConstraintBox<'_, T>;
    // fn advance(&mut self) {}
}

impl<T, F> Fact<T> for Vec<F>
where
    T: 'static + Bounds,
    F: Fact<T>,
{
    fn constraint(&mut self) -> ConstraintBox<'_, T> {
        Box::new(self.iter_mut().map(|f| f.constraint()).collect::<Vec<_>>())
    }
}

pub struct SimpleFact<T, C>(C, PhantomData<T>);

impl<T, C> SimpleFact<T, C> {
    pub fn new(c: C) -> Self {
        SimpleFact(c, PhantomData)
    }
}

impl<T, C> Fact<T> for SimpleFact<T, C>
where
    T: Bounds,
    C: Constraint<T> + Clone,
{
    fn constraint(&mut self) -> ConstraintBox<'_, T> {
        Box::new(self.0.clone())
    }
}

/// Check that all of the constraints of all Facts are satisfied for this sequence.
#[tracing::instrument(skip(fact))]
pub fn check_seq<O, F>(seq: &[O], mut fact: F)
where
    F: Fact<O>,
    O: Bounds,
{
    for (_i, obj) in seq.iter().enumerate() {
        tracing::trace!("i: {}", _i);
        fact.constraint().check(obj)
    }
}

/// Build a sequence from scratch such that all Facts are satisfied.
#[tracing::instrument(skip(u, fact))]
pub fn build_seq<O, F>(u: &mut Unstructured<'static>, num: usize, mut fact: F) -> Vec<O>
where
    O: Arbitrary<'static> + Bounds,
    F: Fact<O>,
{
    let mut seq = Vec::new();
    for _i in 0..num {
        let mut obj = O::arbitrary(u).unwrap();
        tracing::trace!("i: {}", _i);
        fact.constraint().mutate(&mut obj, u);
        seq.push(obj);
    }
    return seq;
}
