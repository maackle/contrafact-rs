use arbitrary::*;
use predicates::contrafact::Constraint;
use std::fmt::Debug;

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
