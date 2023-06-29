use std::{marker::PhantomData, sync::Arc};

use crate::*;

/// Lifts a Fact about a subset of some data into a Fact about the superset.
///
/// In other words, if type `O` contains a `T`, and you have a `Fact<'a, T>`,
/// `LensFact` lets you lift that fact into a `Fact<'a, O>`.
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
/// let mut g = utils::random_generator();
/// let a = fact.build(&mut g).unwrap();
/// assert_eq!(a.x, 1);
/// ```
//
// TODO: can rewrite this in terms of PrismFact for DRYness
pub fn lens<'a, O, T, F, L, S>(label: S, lens: L, inner_fact: F) -> LensFact<'a, O, T, F>
where
    O: Bounds<'a>,
    T: Bounds<'a> + Clone,
    S: ToString,
    F: Fact<'a, T>,
    L: 'a + Clone + Send + Sync + Fn(&mut O) -> &mut T,
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
pub struct LensFact<'a, O, T, F>
where
    T: Bounds<'a>,
    O: Bounds<'a>,
    F: Fact<'a, T>,
{
    label: String,

    getter: Arc<dyn 'a + Send + Sync + Fn(O) -> T>,
    setter: Arc<dyn 'a + Send + Sync + Fn(O, T) -> O>,

    /// The inner_fact about the inner substructure
    inner_fact: F,

    __phantom: PhantomData<&'a F>,
}

impl<'a, O, T, F> LensFact<'a, O, T, F>
where
    T: Bounds<'a>,
    O: Bounds<'a>,
    F: Fact<'a, T>,
{
    /// Constructor. Supply a lens and an existing Fact to create a new Fact.
    pub fn new<L, G, S>(label: L, getter: G, setter: S, inner_fact: F) -> Self
    where
        T: Bounds<'a>,
        O: Bounds<'a>,
        F: Fact<'a, T>,
        L: ToString,
        G: 'a + Send + Sync + Fn(O) -> T,
        S: 'a + Send + Sync + Fn(O, T) -> O,
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

impl<'a, O, T, F> Fact<'a, O> for LensFact<'a, O, T, F>
where
    T: Bounds<'a>,
    O: Bounds<'a> + Clone,
    F: Fact<'a, T>,
{
    #[tracing::instrument(fields(fact = "lens"), skip(self, g))]
    fn mutate(&self, obj: O, g: &mut Generator<'a>) -> Mutation<O> {
        let t = (self.getter)(obj.clone());
        let t = self
            .inner_fact
            .mutate(t, g)
            .map_check_err(|err| format!("lens({}) > {}", self.label, err))?;
        Ok((self.setter)(obj, t))
    }

    #[tracing::instrument(fields(fact = "lens"), skip(self))]
    fn advance(&mut self, obj: &O) {
        self.inner_fact.advance(&(self.getter)(obj.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{eq, utils};
    use arbitrary::*;

    #[derive(Debug, Clone, PartialEq, Arbitrary)]
    struct S {
        x: u32,
        y: u32,
    }

    #[test]
    fn test() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let f = || {
            seq(
                "list of ones",
                lens("S::x", |s: &mut S| &mut s.x, eq("must be 1", 1)),
            )
        };
        let ones = f().build(&mut g);
        f().check(&ones).unwrap();

        assert!(ones.iter().all(|s| s.x == 1));
    }
}
