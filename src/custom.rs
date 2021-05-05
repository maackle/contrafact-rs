use std::sync::Arc;

use arbitrary::Unstructured;

use crate::{
    constraint::{Bounds, CheckResult},
    Constraint,
};

/// A constraint defined by a custom predicate closure.
///
/// NOTE: When doing mutationation, this constraint can do no better than
/// brute force when finding data that satisfies the constraint. Therefore,
/// if the predicate is unlikely to return `true` given arbitrary data,
/// this constraint is a bad choice!
///
/// ALSO NOTE: It is probably best to place this constraint at the beginning
/// of a chain when doing mutation, because if the closure specifies a weak
/// constraint, the mutation may drastically alter the data, potentially undoing
/// constraints that were met by previous mutations.
///
/// There is a fixed iteration limit, beyond which this will panic.
pub fn custom<T, F, S>(reason: S, f: F) -> Box<CustomConstraint<'static, T>>
where
    S: ToString,
    T: Bounds,
    F: 'static + Fn(&T) -> bool,
{
    Box::new(CustomConstraint::new(reason.to_string(), f))
}

#[derive(Clone)]
pub struct CustomConstraint<'a, T> {
    reason: String,
    f: Arc<dyn 'a + Fn(&T) -> bool>,
}

impl<'a, T> Constraint<T> for CustomConstraint<'a, T>
where
    T: Bounds,
{
    fn check(&self, t: &T) -> CheckResult {
        if (self.f)(t) {
            Vec::with_capacity(0)
        } else {
            vec![self.reason.clone()]
        }
        .into()
    }

    fn mutate(&mut self, t: &mut T, u: &mut Unstructured<'static>) {
        const ITERATION_LIMIT: usize = 100;

        for _ in 0..ITERATION_LIMIT {
            if (self.f)(t) {
                return;
            }
            *t = T::arbitrary(u).unwrap();
        }

        panic!(
            "Exceeded iteration limit of {} while attempting to meet a CustomConstraint",
            ITERATION_LIMIT
        );
    }
}

impl<'a, T> CustomConstraint<'a, T> {
    pub(crate) fn new<C: 'a + Fn(&T) -> bool>(reason: String, f: C) -> Self {
        Self {
            reason,
            f: Arc::new(f),
        }
    }
}
