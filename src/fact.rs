use arbitrary::*;

use crate::Check;

/// When running `Fact::satisfy`, repeat mutate+check this many times, in case
/// repetition helps ease into the constraint.
pub(crate) const SATISFY_ATTEMPTS: usize = 3;

/// The trait bounds for the subject of a Fact
pub trait Bounds<'a>: std::fmt::Debug + PartialEq + Arbitrary<'a> {}
impl<'a, T> Bounds<'a> for T where T: std::fmt::Debug + PartialEq + Arbitrary<'a> {}

/// Type alias for a boxed Fact. Implements [`Fact`] itself.
pub type BoxFact<'a, T> = Box<dyn 'a + Fact<'a, T>>;

/// Type alias for a Vec of boxed Facts. Implements [`Fact`] itself.
pub type Facts<'a, T> = Vec<BoxFact<'a, T>>;

/// A declarative representation of a constraint on some data, which can be
/// used to both make an assertion (check) or to mold some arbitrary existing
/// data into a shape which passes that same assertion (mutate)
pub trait Fact<'a, T>
where
    T: Bounds<'a>,
{
    /// Assert that the constraint is satisfied (panic if not).
    fn check(&self, obj: &T) -> Check;

    /// Apply a mutation which moves the obj closer to satisfying the overall
    /// constraint.
    #[cfg(feature = "mutate-inplace")]
    fn mutate(&self, obj: &mut T, u: &mut Unstructured<'a>);

    /// Apply a mutation which moves the obj closer to satisfying the overall
    /// constraint.
    #[cfg(feature = "mutate-owned")]
    fn mutate(&self, obj: T, u: &mut Unstructured<'a>) -> T;

    /// When checking or mutating a sequence of items, this gets called after
    /// each item to modify the state to get ready for the next item.
    fn advance(&mut self, obj: &T);

    /// Mutate a value such that it satisfies the constraint.
    /// If the constraint cannot be satisfied, panic.
    fn satisfy(&mut self, mut obj: T, u: &mut Unstructured<'a>) -> T {
        let mut last_failure: Vec<String> = vec![];
        for _i in 0..SATISFY_ATTEMPTS {
            obj = self.mutate(obj, u);
            if let Err(errs) = self.check(&obj).result() {
                last_failure = errs;
            } else {
                return obj;
            }
        }
        panic!(
            "Could not satisfy a constraint even after {} iterations. Last check failure: {:?}",
            SATISFY_ATTEMPTS, last_failure
        );
    }

    /// Build a new value such that it satisfies the constraint
    fn build(&mut self, u: &mut Unstructured<'a>) -> T {
        let obj = T::arbitrary(u).unwrap();
        self.satisfy(obj, u)
    }
}

impl<'a, T, F> Fact<'a, T> for Box<F>
where
    T: Bounds<'a>,
    F: Fact<'a, T> + ?Sized,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, obj: &T) -> Check {
        tracing::trace!("check");
        (*self).as_ref().check(obj)
    }

    #[tracing::instrument(skip(self, u))]
    #[cfg(feature = "mutate-inplace")]
    fn mutate(&self, obj: &mut T, u: &mut Unstructured<'a>) {
        (*self).as_ref().mutate(obj, u)
    }

    #[tracing::instrument(skip(self, u))]
    #[cfg(feature = "mutate-owned")]
    fn mutate(&self, obj: T, u: &mut Unstructured<'a>) -> T {
        (*self).as_ref().mutate(obj, u)
    }

    #[tracing::instrument(skip(self))]
    fn advance(&mut self, obj: &T) {
        (*self).as_mut().advance(obj)
    }
}

impl<'a, T, F> Fact<'a, T> for &mut [F]
where
    T: Bounds<'a>,
    F: Fact<'a, T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, obj: &T) -> Check {
        self.iter()
            .flat_map(|f| f.check(obj))
            .collect::<Vec<_>>()
            .into()
    }

    #[tracing::instrument(skip(self, u))]
    #[cfg(feature = "mutate-inplace")]
    fn mutate(&self, mut obj: &mut T, u: &mut Unstructured<'a>) {
        for f in self.iter() {
            obj = f.mutate(obj, u);
        }
        obj
    }

    #[tracing::instrument(skip(self, u))]
    #[cfg(feature = "mutate-owned")]
    fn mutate(&self, mut obj: T, u: &mut Unstructured<'a>) -> T {
        for f in self.iter() {
            obj = f.mutate(obj, u);
        }
        obj
    }

    #[tracing::instrument(skip(self))]
    fn advance(&mut self, obj: &T) {
        for f in self.iter_mut() {
            f.advance(obj)
        }
    }
}

impl<'a, T, F> Fact<'a, T> for Vec<F>
where
    T: Bounds<'a>,
    F: Fact<'a, T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, obj: &T) -> Check {
        self.iter()
            .flat_map(|f| f.check(obj))
            .collect::<Vec<_>>()
            .into()
    }

    #[tracing::instrument(skip(self, u))]
    #[cfg(feature = "mutate-inplace")]
    fn mutate(&self, mut obj: &mut T, u: &mut Unstructured<'a>) {
        for f in self.iter() {
            obj = f.mutate(obj, u);
        }
        obj
    }

    #[tracing::instrument(skip(self, u))]
    #[cfg(feature = "mutate-owned")]
    fn mutate(&self, mut obj: T, u: &mut Unstructured<'a>) -> T {
        for f in self.iter() {
            obj = f.mutate(obj, u);
        }
        obj
    }

    #[tracing::instrument(skip(self))]
    fn advance(&mut self, obj: &T) {
        for f in self.iter_mut() {
            f.advance(obj)
        }
    }
}
