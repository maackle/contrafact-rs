use std::{marker::PhantomData, sync::Arc};

use crate::fact::*;
use arbitrary::Unstructured;

/// Applies a Fact to a subset of some data by means of a lens-like closure
/// which specifies the mutable subset to operate on. In other words, if type `O`
/// contains a `T`, and you have a `Fact<T>`, `LensFact` lets you lift
/// that constraint into a constraint about `O`.
//
// TODO: can rewrite this in terms of PrismFact for DRYness
pub fn lens<O, T, F, L, S>(reason: S, lens: L, constraint: F) -> LensFact<O, T, F>
where
    O: Bounds,
    T: Bounds,
    S: ToString,
    F: Fact<T>,
    L: 'static + Fn(&mut O) -> &mut T,
{
    LensFact::new(reason.to_string(), lens, constraint)
}

#[derive(Clone)]
pub struct LensFact<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Fact<T>,
{
    reason: String,

    /// Function which maps outer structure to inner substructure
    lens: Arc<dyn 'static + Fn(&mut O) -> &mut T>,

    /// The constraint about the inner substructure
    constraint: F,

    __phantom: PhantomData<F>,
}

impl<O, T, F> LensFact<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Fact<T>,
{
    /// Constructor. Supply a lens and an existing Fact to create a new Fact.
    pub fn new<L>(reason: String, lens: L, constraint: F) -> Self
    where
        T: Bounds,
        O: Bounds,
        F: Fact<T>,
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

impl<O, T, F> Fact<O> for LensFact<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Fact<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&mut self, o: &O) -> CheckResult {
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

        let f = || lens("S::x", |s: &mut S| &mut s.x, predicate::eq("must be 1", &1));

        let ones = build_seq(&mut u, 3, f());
        check_seq(ones.as_slice(), f()).unwrap();

        assert!(ones.iter().all(|s| s.x == 1));
    }
}
