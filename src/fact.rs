use std::marker::PhantomData;

use crate::{
    constraint::{Bounds, CheckResult, ConstraintBox},
    Constraint,
};
use arbitrary::{Arbitrary, Unstructured};

/// A "Fact" is simply a data type which can produce a `Constraint`.
/// When producing the constraint, there is an opportunity to mutate the Fact's
/// state, causing it to produce a different Constraint next time `.constraint`
/// is called. This allows for `fold()`-like application of a Fact to a sequence
/// of data.
pub trait Fact<T> {
    /// Produce the constraint given the current state of this Fact
    fn constraint(&mut self) -> ConstraintBox<'_, T>;
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

#[derive(Clone, Debug)]
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
pub fn check_seq<O, F>(seq: &[O], mut fact: F) -> CheckResult
where
    F: Fact<O>,
    O: Bounds,
{
    let mut reasons: Vec<String> = Vec::new();
    for (i, obj) in seq.iter().enumerate() {
        reasons.extend(
            fact.constraint()
                .check(obj)
                .into_iter()
                .map(|reason| format!("item {}: {}", i, reason))
                .collect::<Vec<_>>(),
        );
    }
    reasons.into()
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

/// Convenience macro for creating a collection of `Facts`s of different types.
/// The resulting value also implements `Fact`.
#[macro_export]
macro_rules! facts {
    ( $( $fact:expr ,)+ ) => {{
        let mut fs = Vec::new();
        $(
            fs.push(Box::new($fact));
        )+
        fs
    }};
}
