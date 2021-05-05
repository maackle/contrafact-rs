use std::{marker::PhantomData, sync::Arc};

use crate::constraint::*;
use arbitrary::Unstructured;

pub fn prism<O, T, C, P>(prism: P, constraint: C) -> Box<PrismConstraint<O, T, C>>
where
    O: Bounds,
    T: Bounds,
    C: Constraint<T>,
    P: 'static + Fn(&mut O) -> Option<&mut T>,
{
    Box::new(PrismConstraint::new(prism, constraint))
}

#[derive(Clone)]
/// Applies a Constraint to a subset of some data by means of a prism-like closure
/// which specifies the mutable subset to operate on. In other words, if type `O`
/// contains a `T`, and you have a `Constraint<T>`, `PrismConstraint` lets you lift that constraint
/// into a constraint about `O`.
pub struct PrismConstraint<O, T, C>
where
    T: Bounds,
    O: Bounds,
    C: Constraint<T>,
{
    /// Function which maps outer structure to inner substructure
    pub(crate) prism: Arc<dyn 'static + Fn(&mut O) -> Option<&mut T>>,

    /// The constraint about the inner substructure
    pub(crate) constraint: C,

    __phantom: PhantomData<C>,
}

impl<O, T, C> PrismConstraint<O, T, C>
where
    T: Bounds,
    O: Bounds,
    C: Constraint<T>,
{
    /// Constructor. Supply a prism and an existing Constraint to create a new Constraint.
    pub fn new<P>(prism: P, constraint: C) -> Self
    where
        T: Bounds,
        O: Bounds,
        C: Constraint<T>,
        P: 'static + Fn(&mut O) -> Option<&mut T>,
    {
        Self {
            prism: Arc::new(prism),
            constraint,
            __phantom: PhantomData,
        }
    }
}

impl<O, T, C> Constraint<O> for PrismConstraint<O, T, C>
where
    T: Bounds,
    O: Bounds,
    C: Constraint<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, o: &O) {
        unsafe {
            // We can convert the immutable ref to a mutable one because `check`
            // never mutates the value, but we need `prism` to return a mutable
            // reference so it can be reused in `mutate`
            let o = o as *const O;
            let o = o as *mut O;
            if let Some(t) = (self.prism)(&mut *o) {
                self.constraint.check(t)
            }
        }
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut O, u: &mut Unstructured<'static>) {
        if let Some(t) = (self.prism)(obj) {
            self.constraint.mutate(t, u)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate;
    use crate::{build_seq, check_seq, NOISE};
    use arbitrary::*;

    #[derive(Debug, Clone, PartialEq, Arbitrary)]
    enum E {
        X(u32),
        Y(u32),
    }

    impl E {
        fn x(&mut self) -> Option<&mut u32> {
            match self {
                E::X(x) => Some(x),
                _ => None,
            }
        }
        fn y(&mut self) -> Option<&mut u32> {
            match self {
                E::Y(y) => Some(y),
                _ => None,
            }
        }
    }

    #[test]
    fn test() {
        observability::test_run().ok();
        let mut u = Unstructured::new(&NOISE);

        let f = || vec![prism(E::x, predicate::eq(1)), prism(E::y, predicate::eq(2))].to_fact();

        let seq = build_seq(&mut u, 6, f());
        check_seq(seq.as_slice(), f());

        assert!(seq.iter().all(|e| match e {
            E::X(x) => *x == 1,
            E::Y(y) => *y == 2,
        }))
    }
}
