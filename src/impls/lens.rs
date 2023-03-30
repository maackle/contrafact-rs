use std::{marker::PhantomData, sync::Arc};

use crate::{fact::*, Check};
use arbitrary::Unstructured;

/// Lifts a Fact about a subset of some data into a Fact about the superset.
///
/// In other words, if type `O` contains a `T`, and you have a `Fact<T>`,
/// `LensFact` lets you lift that fact into a `Fact<O>`.
///
/// The `lens` closure provides a mutable view into the subset of data.
/// There must be a way to specify a mutable reference to the subset of data.
/// If this is not always possible, consider using [`prism()`](crate::prism) instead.
///
/// This is a lazy way to provide a lens in the traditional optics sense.
/// We may consider using a true lens library for this in the future.
///
/// ```
/// use contrafact::*;
/// use arbitrary::*;
///
/// #[derive(Debug, Clone, PartialEq, Arbitrary)]
/// struct S {
///     x: u32,
///     y: u32,
/// }
///
/// let mut fact = lens("S::x", |s: &mut S| &mut s.x, eq("must be 1", 1));
///
/// assert!(fact.check(&S {x: 1, y: 333}).is_ok());
/// assert!(fact.check(&S {x: 2, y: 333}).is_err());
///
/// let mut u = Unstructured::new(&[0; 9999]);
/// let a = fact.build(&mut u);
/// assert_eq!(a.x, 1);
/// ```
//
// TODO: can rewrite this in terms of PrismFact for DRYness
pub fn lens<O, T, F, L, S>(label: S, lens: L, inner_fact: F) -> LensFact<O, T, F>
where
    O: Bounds,
    T: Bounds + Clone,
    S: ToString,
    F: Fact<T>,
    L: 'static + Clone + Fn(&mut O) -> &mut T,
{
    let lens2 = lens.clone();
    let getter = move |mut o| lens(&mut o).clone();
    let setter = move |mut o, t: T| {
        let r = lens2(&mut o);
        *r = t;
        o
    };
    LensFact::new(label.to_string(), getter, setter, inner_fact)
}

/// A fact which uses a lens to apply another fact. Use [`lens()`] to construct.
#[derive(Clone)]
pub struct LensFact<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Fact<T>,
{
    label: String,

    getter: Arc<dyn 'static + Fn(O) -> T>,
    setter: Arc<dyn 'static + Fn(O, T) -> O>,

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
    pub fn new<L, G, S>(label: L, getter: G, setter: S, inner_fact: F) -> Self
    where
        T: Bounds,
        O: Bounds,
        F: Fact<T>,
        L: ToString,
        G: 'static + Fn(O) -> T,
        S: 'static + Fn(O, T) -> O,
    {
        Self {
            label: label.to_string(),
            getter: Arc::new(getter),
            setter: Arc::new(setter),
            inner_fact,
            __phantom: PhantomData,
        }
    }
}

impl<O, T, F> Fact<O> for LensFact<O, T, F>
where
    T: Bounds,
    O: Bounds + Clone,
    F: Fact<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, obj: &O) -> Check {
        self.inner_fact
            .check(&(self.getter)(obj.clone()))
            .map(|err| format!("lens({}) > {}", self.label, err))
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&self, obj: O, u: &mut Unstructured<'static>) -> O {
        let t = (self.getter)(obj.clone());
        let t = self.inner_fact.mutate(t, u);
        (self.setter)(obj, t)
    }

    #[tracing::instrument(skip(self))]
    fn advance(&mut self, obj: &O) {
        self.inner_fact.advance(&(self.getter)(obj.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_seq, check_seq, eq, utils};
    use arbitrary::*;

    #[derive(Debug, Clone, PartialEq, Arbitrary)]
    struct S {
        x: u32,
        y: u32,
    }

    #[test]
    fn test() {
        observability::test_run().ok();
        let mut u = utils::unstructured_noise();

        let f = || lens("S::x", |s: &mut S| &mut s.x, eq("must be 1", 1));

        let ones = build_seq(&mut u, 3, f());
        check_seq(ones.as_slice(), f()).unwrap();

        assert!(ones.iter().all(|s| s.x == 1));
    }
}