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
        T: Clone + Eq + Debug + Arbitrary<'static>,
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

    /// Add a new constraint via a check/mutate pair.
    /// It's preferred to use `add` when possible, which is more composable and
    /// offers less chance of mismatch beteen the check and mutate implementations.
    /// Use this only when unable to use `add`.
    pub fn add_pair<T, FC, FM>(&mut self, check: FC, mutate: FM)
    where
        FC: 'static + Fn(&mut O),
        FM: 'static + Fn(&mut O, &mut Unstructured<'static>),
    {
        self.checks.push(Box::new(check));
        self.mutations.push(Box::new(mutate));
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
pub struct PredicateConstraint<'a, T>(Arc<dyn 'a + Fn(&T) -> bool>);

impl<'a, T> Constraint<T> for PredicateConstraint<'a, T>
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

impl<'a, T> PredicateConstraint<'a, T> {
    pub fn new<F: 'a + Fn(&T) -> bool>(f: F) -> Self {
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
pub fn predicate<'a, T, F: 'a + Fn(&T) -> bool>(f: F) -> PredicateConstraint<'a, T> {
    PredicateConstraint::new(f)
}
