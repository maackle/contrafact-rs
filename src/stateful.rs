use std::marker::PhantomData;

use crate::constraint::*;
use arbitrary::*;

pub fn stateful<S, T, F>(state: S, fact: F) -> Box<StatefulFact<S, T, F>>
where
    T: Bounds,
    F: Constraint<T>,
{
    Box::new(StatefulFact::new(state, fact))
}

pub struct StatefulFact<S, T, F> {
    state: S,
    fact: F,
    __phantom: PhantomData<T>,
}

impl<S, T, F> StatefulFact<S, T, F>
where
    T: Bounds,
    F: Constraint<T>,
{
    pub fn new(state: S, fact: F) -> Self {
        Self {
            state,
            fact,
            __phantom: PhantomData,
        }
    }
}
