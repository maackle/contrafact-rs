use arbitrary::*;
use predicates::contrafact::Constraint;
use std::{fmt::Debug, sync::Arc};

/// A set of declarative constraints. You can add to this set with `add`.
///
/// This type is a bit of a trick, meant to hide away the details of the various
/// constraints it contains. In general, the Constraints are about subfields of
/// some containing structure. When adding a Constraint, two closures get
/// immediately created, one for "check" and another for "mutate", which
/// encapsulate the subtype, so that the overall collection needs only be aware
/// of the containing type, `O`.
pub struct Constraints<O> {
    /// Closures which run assertions on the object.
    pub(crate) checks: Vec<Box<dyn 'static + Fn(&mut O)>>,
    /// Closures which perform mutations on the object.
    pub(crate) mutations: Vec<Box<dyn 'static + Fn(&mut O, &mut Unstructured<'static>)>>,
}

impl<O> Constraints<O> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            mutations: Vec::new(),
        }
    }

    /// Add a new constraint. This generates two functions,
    /// a "check" and a "mutation", and stores them in the Constraints'
    /// internal state.
    pub fn add<T, G, C>(&mut self, get: G, constraint: C)
    where
        T: 'static + Clone + Eq + Debug + Arbitrary<'static>,
        G: 'static + Clone + Fn(&mut O) -> &mut T,
        C: 'static + Constraint<T>,
    {
        let g = get.clone();
        let constraint2 = constraint.clone();
        self.checks.push(Box::new(move |obj| {
            let t = g(obj);
            constraint.check(t);
        }));
        self.mutations
            .push(Box::new(move |obj, u: &mut Unstructured<'static>| {
                let t = get(obj);
                constraint2.mutate(t, u)
            }));
    }

    /// Combine two sets of Constraints into one
    pub fn extend(&mut self, other: Constraints<O>) {
        self.checks.extend(other.checks.into_iter());
        self.mutations.extend(other.mutations.into_iter());
    }
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
pub struct PredicateConstraint<T>(Arc<dyn Fn(&T) -> bool>);

impl<T> Constraint<T> for PredicateConstraint<T>
where
    T: predicates::contrafact::Bounds,
{
    fn check(&self, t: &T) {
        assert!(self.0(t))
    }

    fn mutate(&self, t: &mut T, u: &mut Unstructured<'static>) {
        const ITERATION_LIMIT: usize = 100;

        for _ in 0..ITERATION_LIMIT {
            *t = T::arbitrary(u).unwrap();
            if self.0(t) {
                return;
            }
        }

        panic!(
            "Exceeded iteration limit of {} while attempting to meet a PredicateConstraint",
            ITERATION_LIMIT
        );
    }
}

impl<T> PredicateConstraint<T> {
    pub fn new<F: 'static + Fn(&T) -> bool>(f: F) -> Self {
        Self(Arc::new(f))
    }
}

/// A constraint defined by a custom predicate closure.
///
/// NOTE: When using during a mutation, this type can do no better than
/// brute force when finding data that matches the constraint. Therefore,
/// if the predicate is unlikely to return `true` given arbitrary data,
/// this constraint is a bad choice!
///
/// There is a fixed iteration limit, beyond which this will panic.
pub fn predicate<T, F: 'static + Fn(&T) -> bool>(f: F) -> PredicateConstraint<T> {
    PredicateConstraint::new(f)
}
