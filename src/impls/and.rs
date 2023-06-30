use std::marker::PhantomData;

use crate::*;

/// Fact that combines two `Fact`s, returning the OR of the results.
///
/// This is created by the `or` function.
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
    fn mutate(&self, obj: T, g: &mut Generator<'a>) -> Mutation<T> {
        let obj = self.a.mutate(obj, g)?;
        let obj = self.b.mutate(obj, g)?;
        Ok(obj)
    }

    fn advance(&mut self, obj: &T) {
        self.a.advance(obj);
        self.b.advance(obj);
    }
}
