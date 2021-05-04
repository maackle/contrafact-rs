use std::marker::PhantomData;

use crate::constraint::*;
use arbitrary::Unstructured;

pub fn lens<O, T, F, L>(lens: L, fact: F) -> Box<LensConstraint<O, T, F>>
where
    O: Bounds,
    T: Bounds,
    F: Constraint<T>,
    L: 'static + Fn(&mut O) -> &mut T,
{
    Box::new(LensConstraint::new(lens, fact))
}

/// Applies a Constraint to a subset of some data by means of a lens-like closure
/// which specifies the mutable subset to operate on. In other words, if type `O`
/// contains a `T`, and you have a `Constraint<T>`, `LensConstraint` lets you lift that fact
/// into a fact about `O`.
pub struct LensConstraint<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Constraint<T>,
{
    /// Function which maps outer structure to inner substructure
    pub(crate) lens: Box<dyn 'static + Fn(&mut O) -> &mut T>,

    /// The fact about the inner substructure
    pub(crate) fact: F,

    __phantom: PhantomData<F>,
}

impl<O, T, F> LensConstraint<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Constraint<T>,
{
    /// Constructor. Supply a lens and an existing Constraint to create a new Constraint.
    pub fn new<L>(lens: L, fact: F) -> Self
    where
        T: Bounds,
        O: Bounds,
        F: Constraint<T>,
        L: 'static + Fn(&mut O) -> &mut T,
    {
        Self {
            lens: Box::new(lens),
            fact,
            __phantom: PhantomData,
        }
    }
}

impl<O, T, F> Constraint<O> for LensConstraint<O, T, F>
where
    T: Bounds,
    O: Bounds,
    F: Constraint<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, o: &O) {
        unsafe {
            // We can convert the immutable ref to a mutable one because `check`
            // never mutates the value, but we need `lens` to return a mutable
            // reference so it can be reused in `mutate`
            let o = o as *const O;
            let o = o as *mut O;
            self.fact.check((self.lens)(&mut *o))
        }
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut O, u: &mut Unstructured<'static>) {
        self.fact.mutate((self.lens)(obj), u)
    }
}
