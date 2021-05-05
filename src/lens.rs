use std::{marker::PhantomData, sync::Arc};

use crate::constraint::*;
use arbitrary::Unstructured;

/// Convenient constructor for LensConstraint
pub fn lens<O, T, C, L, S>(reason: S, lens: L, constraint: C) -> Box<LensConstraint<O, T, C>>
where
    O: Bounds,
    T: Bounds,
    S: ToString,
    C: Constraint<T>,
    L: 'static + Fn(&mut O) -> &mut T,
{
    Box::new(LensConstraint::new(reason.to_string(), lens, constraint))
}

#[derive(Clone)]
/// Applies a Constraint to a subset of some data by means of a lens-like closure
/// which specifies the mutable subset to operate on. In other words, if type `O`
/// contains a `T`, and you have a `Constraint<T>`, `LensConstraint` lets you lift
/// that constraint into a constraint about `O`.
//
// TODO: can rewrite this in terms of PrismConstraint for DRYness
pub struct LensConstraint<O, T, C>
where
    T: Bounds,
    O: Bounds,
    C: Constraint<T>,
{
    reason: String,

    /// Function which maps outer structure to inner substructure
    lens: Arc<dyn 'static + Fn(&mut O) -> &mut T>,

    /// The constraint about the inner substructure
    constraint: C,

    __phantom: PhantomData<C>,
}

impl<O, T, C> LensConstraint<O, T, C>
where
    T: Bounds,
    O: Bounds,
    C: Constraint<T>,
{
    /// Constructor. Supply a lens and an existing Constraint to create a new Constraint.
    pub fn new<L>(reason: String, lens: L, constraint: C) -> Self
    where
        T: Bounds,
        O: Bounds,
        C: Constraint<T>,
        L: 'static + Fn(&mut O) -> &mut T,
    {
        Self {
            reason,
            lens: Arc::new(lens),
            constraint,
            __phantom: PhantomData,
        }
    }
}

impl<O, T, C> Constraint<O> for LensConstraint<O, T, C>
where
    T: Bounds,
    O: Bounds,
    C: Constraint<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, o: &O) -> CheckResult {
        unsafe {
            // We can convert the immutable ref to a mutable one because `check`
            // never mutates the value, but we need `lens` to return a mutable
            // reference so it can be reused in `mutate`
            let o = o as *const O;
            let o = o as *mut O;
            self.constraint
                .check((self.lens)(&mut *o))
                .into_iter()
                .map(|err| format!("lens {} > {}", self.reason, err))
                .collect::<Vec<_>>()
                .into()
        }
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut O, u: &mut Unstructured<'static>) {
        self.constraint.mutate((self.lens)(obj), u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate;
    use crate::{build_seq, check_seq, NOISE};
    use arbitrary::*;

    #[derive(Debug, Clone, PartialEq, Arbitrary)]
    struct S {
        x: u32,
        y: u32,
    }

    #[test]
    fn test() {
        observability::test_run().ok();
        let mut u = Unstructured::new(&NOISE);

        let f = || lens("S::x", |s: &mut S| &mut s.x, predicate::eq("must be 1", 1)).to_fact();

        let ones = build_seq(&mut u, 3, f());
        check_seq(ones.as_slice(), f()).unwrap();

        assert!(ones.iter().all(|s| s.x == 1));
    }
}
