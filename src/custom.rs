use std::sync::Arc;

use arbitrary::Unstructured;

use crate::{
    fact::{Bounds, CheckResult},
    Fact,
};

pub(crate) const ITERATION_LIMIT: usize = 100;

/// A constraint defined only by a custom predicate closure.
/// This is appropriate to use when the space of possible values is small, and
/// you can rely on randomness to eventually find a value that matches the
/// constraint, e.g. when requiring a particular enum variant.
///
/// NOTE: When doing mutation, this constraint can do no better than
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
pub fn custom<T, F, S>(reason: S, f: F) -> CustomFact<'static, T>
where
    S: ToString,
    T: Bounds,
    F: 'static + Fn(&T) -> bool,
{
    CustomFact::<'static, T>::new(reason.to_string(), f)
}

#[derive(Clone)]
pub struct CustomFact<'a, T> {
    reason: String,
    f: Arc<dyn 'a + Fn(&T) -> bool>,
}

impl<'a, T> Fact<T> for CustomFact<'a, T>
where
    T: Bounds,
{
    fn check(&mut self, t: &T) -> CheckResult {
        if (self.f)(t) {
            Vec::with_capacity(0)
        } else {
            vec![self.reason.clone()]
        }
        .into()
    }

    fn mutate(&mut self, t: &mut T, u: &mut Unstructured<'static>) {
        for _ in 0..ITERATION_LIMIT {
            if (self.f)(t) {
                return;
            }
            *t = T::arbitrary(u).unwrap();
        }

        panic!(
            "Exceeded iteration limit of {} while attempting to meet a CustomFact",
            ITERATION_LIMIT
        );
    }
}

impl<'a, T> CustomFact<'a, T> {
    pub(crate) fn new<F: 'a + Fn(&T) -> bool>(reason: String, f: F) -> Self {
        Self {
            reason,
            f: Arc::new(f),
        }
    }
}
