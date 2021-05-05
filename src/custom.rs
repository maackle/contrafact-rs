use std::sync::Arc;

use arbitrary::Unstructured;

use crate::{constraint::Bounds, Constraint};

pub fn custom<T, F>(f: F) -> Box<CustomConstraint<'static, T>>
where
    T: Bounds,
    F: 'static + Fn(&T) -> bool,
{
    Box::new(CustomConstraint::new(f))
}

/// A constraint defined by a custom predicate closure.
///
/// NOTE: When using during a mutation, this type can do no better than
/// brute force when finding data that matches the constraint. Therefore,
/// if the predicate is unlikely to return `true` given arbitrary data,
/// this constraint is a bad choice!
///
/// There is a fixed iteration limit, beyond which this will panic.
#[derive(Clone)]
pub struct CustomConstraint<'a, T>(Arc<dyn 'a + Fn(&T) -> bool>);

impl<'a, T> Constraint<T> for CustomConstraint<'a, T>
where
    T: Bounds,
{
    fn check(&self, t: &T) {
        assert!(self.0(t))
    }

    fn mutate(&mut self, t: &mut T, u: &mut Unstructured<'static>) {
        const ITERATION_LIMIT: usize = 100;

        for _ in 0..ITERATION_LIMIT {
            *t = T::arbitrary(u).unwrap();
            if self.0(t) {
                return;
            }
        }

        panic!(
            "Exceeded iteration limit of {} while attempting to meet a CustomConstraint",
            ITERATION_LIMIT
        );
    }
}

impl<'a, T> CustomConstraint<'a, T> {
    pub fn new<C: 'a + Fn(&T) -> bool>(f: C) -> Self {
        Self(Arc::new(f))
    }
}
