use arbitrary::*;

use crate::fact::SimpleFact;

/// The trait bounds for the subject of a Constraint
pub trait Bounds: std::fmt::Debug + PartialEq + Arbitrary<'static> + Clone {}
impl<T> Bounds for T where T: std::fmt::Debug + PartialEq + Arbitrary<'static> + Clone {}

pub trait ConstraintSized<T>: Constraint<T> + Sized
where
    T: Bounds,
{
}
impl<T, C> ConstraintSized<T> for C
where
    T: Bounds,
    C: Constraint<T> + Sized,
{
}

pub type ConstraintBox<'a, T> = Box<dyn 'a + Constraint<T>>;
pub type ConstraintVec<'a, T> = Vec<ConstraintBox<'a, T>>;

/// A declarative representation of a constraint on some data, which can be
/// used to both make an assertion (check) or to mold some aribtrary existing
/// data into a shape which passes that same assertion (mutate)
pub trait Constraint<T>
where
    T: Bounds,
{
    /// Assert that the constraint is satisfied (panic if not).
    fn check(&self, t: &T);

    /// Mutate a value such that it satisfies the constraint.
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>);

    fn to_fact(self) -> SimpleFact<T, Self>
    where
        Self: Sized,
    {
        SimpleFact::new(self)
    }
}

impl<T, C> Constraint<T> for Box<C>
where
    T: Bounds,
    C: Constraint<T> + ?Sized,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, obj: &T) {
        tracing::trace!("check");
        (*self).as_ref().check(obj);
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>) {
        (*self).as_mut().mutate(obj, u);
    }
}

impl<T, C> Constraint<T> for &mut [C]
where
    T: Bounds,
    C: Constraint<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, obj: &T) {
        for f in self.iter() {
            f.check(obj)
        }
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>) {
        for f in self.iter_mut() {
            f.mutate(obj, u)
        }
    }
}

impl<T, C> Constraint<T> for Vec<C>
where
    T: Bounds,
    C: Constraint<T> + Sized,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, obj: &T) {
        for f in self.iter() {
            f.check(obj)
        }
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>) {
        for f in self.iter_mut() {
            f.mutate(obj, u)
        }
    }
}

#[macro_export]
macro_rules! facts {
    ( $( $fact:expr ,)+ ) => {{
        let mut fs = Vec::new();
        $(
            fs.push(Box::new($fact));
        )+
        fs
    }};
}
