use std::{marker::PhantomData, sync::Arc};

use crate::{fact::*, Check};
use arbitrary::Unstructured;

/// Lifts a Fact about some *optional* subset of data into a Fact about the
/// superset.
///
/// In other words, if type `O` contains a `Option<T>`, and you have a `Fact<T>`,
/// `PrismFact` lets you lift that fact into a `Fact<O>`.
///
/// The `prism` closure provides an optional mutable view into the subset.
/// If the prism returns None during any fact application, the fact will
/// effectively be skipped for this item: no check or mutation will be performed,
/// and the state will not advance.
///
/// A prism is like a lens, except that the target value may or may not exist.
/// It is typically used for enums, or any structure where data may or may not
/// be present.
///
/// The `prism` closure is a rather lazy way to provide a prism in the
/// traditional optics sense. We may consider using a true lens library for
/// this in the future.
pub fn prism<O, T, F, P, S>(label: S, prism: P, inner_fact: F) -> PrismFact<O, T, F>
where
    O: Bounds,
    S: ToString,
    T: Bounds,
    F: Fact<T>,
    P: 'static + Fn(&mut O) -> Option<&mut T>,
{
    PrismFact::new(label.to_string(), prism, inner_fact)
}

#[derive(Clone)]
pub struct PrismFact<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Fact<T>,
{
    label: String,
    prism: Arc<dyn 'static + Fn(&mut O) -> Option<&mut T>>,
    inner_fact: F,
    __phantom: PhantomData<F>,
}

impl<O, T, F> PrismFact<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Fact<T>,
{
    /// Constructor. Supply a prism and an existing Fact to create a new Fact.
    pub fn new<P>(label: String, prism: P, inner_fact: F) -> Self
    where
        T: Bounds,
        O: Bounds,
        F: Fact<T>,
        P: 'static + Fn(&mut O) -> Option<&mut T>,
    {
        Self {
            label,
            prism: Arc::new(prism),
            inner_fact,
            __phantom: PhantomData,
        }
    }
}

impl<O, T, F> Fact<O> for PrismFact<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Fact<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, o: &O) -> Check {
        unsafe {
            // We can convert the immutable ref to a mutable one because `check`
            // never mutates the value, but we need `prism` to return a mutable
            // reference so it can be reused in `mutate`
            let o = o as *const O;
            let o = o as *mut O;
            if let Some(t) = (self.prism)(&mut *o) {
                self.inner_fact
                    .check(t)
                    .map(|err| format!("prism({}) > {}", self.label, err))
            } else {
                Vec::with_capacity(0).into()
            }
        }
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&self, obj: &mut O, u: &mut Unstructured<'static>) {
        if let Some(t) = (self.prism)(obj) {
            self.inner_fact.mutate(t, u)
        }
    }

    #[tracing::instrument(skip(self))]
    fn advance(&mut self, obj: &O) {
        unsafe {
            // We can convert the immutable ref to a mutable one because `advance`
            // never mutates the value, but we need `prism` to return a mutable
            // reference so it can be reused in `mutate`
            let o = obj as *const O;
            let o = o as *mut O;
            if let Some(t) = (self.prism)(&mut *o) {
                self.inner_fact.advance(t)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let f = || {
            vec![
                prism("E::x", E::x, crate::eq("must be 1", &1)),
                prism("E::y", E::y, crate::eq("must be 2", &2)),
            ]
        };

        let seq = build_seq(&mut u, 6, f());
        check_seq(seq.as_slice(), f()).unwrap();

        assert!(seq.iter().all(|e| match e {
            E::X(x) => *x == 1,
            E::Y(y) => *y == 2,
        }))
    }

    #[test]
    fn test_stateful_fact_under_prism() {
        todo!()
    }
}
