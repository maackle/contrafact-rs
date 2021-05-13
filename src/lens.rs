use std::{marker::PhantomData, sync::Arc};

use crate::{fact::*, Check};
use arbitrary::Unstructured;

/// Applies a Fact to a subset of some data by means of a lens-like closure
/// which specifies the mutable subset to operate on. In other words, if type `O`
/// contains a `T`, and you have a `Fact<T>`, `LensFact` lets you lift
/// that constraint into a constraint about `O`.
//
// TODO: can rewrite this in terms of PrismFact for DRYness
pub fn lens<O, T, F, L, S>(label: S, lens: L, inner_fact: F) -> LensFact<O, T, F>
where
    O: Bounds,
    T: Bounds,
    S: ToString,
    F: Fact<T>,
    L: 'static + Fn(&mut O) -> &mut T,
{
    LensFact::new(label.to_string(), lens, inner_fact)
}

#[derive(Clone)]
pub struct LensFact<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Fact<T>,
{
    label: String,

    /// Function which maps outer structure to inner substructure
    lens: Arc<dyn 'static + Fn(&mut O) -> &mut T>,

    /// The inner_fact about the inner substructure
    inner_fact: F,

    __phantom: PhantomData<F>,
}

impl<O, T, F> LensFact<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Fact<T>,
{
    /// Constructor. Supply a lens and an existing Fact to create a new Fact.
    pub fn new<L>(label: String, lens: L, inner_fact: F) -> Self
    where
        T: Bounds,
        O: Bounds,
        F: Fact<T>,
        L: 'static + Fn(&mut O) -> &mut T,
    {
        Self {
            label,
            lens: Arc::new(lens),
            inner_fact,
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
    fn check(&self, o: &O) -> Check {
        unsafe {
            // We can convert the immutable ref to a mutable one because `check`
            // never mutates the value, but we need `lens` to return a mutable
            // reference so it can be reused in `mutate`
            let o = o as *const O;
            let o = o as *mut O;
            self.inner_fact
                .check((self.lens)(&mut *o))
                .map(|err| format!("lens({}) > {}", self.label, err))
        }
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&self, obj: &mut O, u: &mut Unstructured<'static>) {
        self.inner_fact.mutate((self.lens)(obj), u)
    }

    #[tracing::instrument(skip(self))]
    fn advance(&mut self) {
        self.inner_fact.advance()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_seq, check_seq, eq, NOISE};
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

        let f = || lens("S::x", |s: &mut S| &mut s.x, eq("must be 1", &1));

        let ones = build_seq(&mut u, 3, f());
        check_seq(ones.as_slice(), f()).unwrap();

        assert!(ones.iter().all(|s| s.x == 1));
    }
}
