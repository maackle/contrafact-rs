use std::marker::PhantomData;

use crate::*;

/// A Fact which applies two other facts.
pub fn and<'a, F1, F2, T>(a: F1, b: F2) -> AndFact<'a, F1, F2, T>
where
    T: Bounds<'a>,
    F1: Fact<'a, T>,
    F2: Fact<'a, T>,
{
    AndFact::new(a, b)
}

/// A fact which applies two facts.
/// This is the primary way to build up a complex fact from simpler facts.
/// The [`facts!`] macro is a more convenient way to compose more than two facts
/// together using [`AndFact`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndFact<'a, F1, F2, T>
where
    F1: Fact<'a, T>,
    F2: Fact<'a, T>,
    T: ?Sized + Bounds<'a>,
{
    pub(crate) a: F1,
    pub(crate) b: F2,
    _phantom: PhantomData<&'a T>,
}

impl<'a, F1, F2, T> AndFact<'a, F1, F2, T>
where
    F1: Fact<'a, T>,
    F2: Fact<'a, T>,
    T: ?Sized + Bounds<'a>,
{
    /// Constructor
    pub fn new(a: F1, b: F2) -> Self {
        Self {
            a,
            b,
            _phantom: PhantomData,
        }
    }
}

impl<'a, F1, F2, T> Fact<'a, T> for AndFact<'a, F1, F2, T>
where
    F1: Fact<'a, T> + Fact<'a, T>,
    F2: Fact<'a, T> + Fact<'a, T>,
    T: Bounds<'a>,
{
    fn mutate(&mut self, g: &mut Generator<'a>, obj: T) -> Mutation<T> {
        let obj = self.a.mutate(g, obj)?;
        let obj = self.b.mutate(g, obj)?;
        Ok(obj)
    }
}
