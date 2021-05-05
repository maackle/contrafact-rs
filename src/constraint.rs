use arbitrary::*;

use crate::fact::SimpleFact;

/// The trait bounds for the subject of a Constraint
pub trait Bounds: std::fmt::Debug + PartialEq + Arbitrary<'static> + Clone {}
impl<T> Bounds for T where T: std::fmt::Debug + PartialEq + Arbitrary<'static> + Clone {}

/// Type alias for a boxed Constraint
pub type ConstraintBox<'a, T> = Box<dyn 'a + Constraint<T>>;

/// Type alias for a Vec of boxed Constraints
pub type ConstraintVec<'a, T> = Vec<ConstraintBox<'a, T>>;

/// The result of a check operation, which contains an error message for every
/// constraint which was not met
#[derive(derive_more::From, derive_more::IntoIterator)]
#[must_use = "CheckResult should be used with either `.unwrap()` or `.ok()`"]
pub struct CheckResult(Vec<String>);

impl CheckResult {
    pub fn unwrap(self) {
        if !self.0.is_empty() {
            let msg = if self.0.len() == 1 {
                format!("Check failed: {}", self.0[0])
            } else {
                format!("Check failed: {:#?}", self.0)
            };
            panic!(msg);
        }
    }

    pub fn ok(self) -> std::result::Result<(), Vec<String>> {
        if self.0.is_empty() {
            std::result::Result::Ok(())
        } else {
            std::result::Result::Err(self.0)
        }
    }

    pub fn pass() -> Self {
        Self(Vec::with_capacity(0))
    }
}

/// A declarative representation of a constraint on some data, which can be
/// used to both make an assertion (check) or to mold some aribtrary existing
/// data into a shape which passes that same assertion (mutate)
pub trait Constraint<T>
where
    T: Bounds,
{
    /// Assert that the constraint is satisfied (panic if not).
    fn check(&self, obj: &T) -> CheckResult;

    /// Mutate a value such that it satisfies the constraint.
    fn mutate(&self, obj: &mut T, u: &mut Unstructured<'static>);

    /// Convert this constraint to a stateless Fact.
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
    fn check(&self, obj: &T) -> CheckResult {
        tracing::trace!("check");
        (*self).as_ref().check(obj)
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&self, obj: &mut T, u: &mut Unstructured<'static>) {
        (*self).as_ref().mutate(obj, u);
    }
}

impl<T, C> Constraint<T> for &mut [C]
where
    T: Bounds,
    C: Constraint<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, obj: &T) -> CheckResult {
        self.iter()
            .flat_map(|f| f.check(obj))
            .collect::<Vec<_>>()
            .into()
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&self, obj: &mut T, u: &mut Unstructured<'static>) {
        for f in self.iter() {
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
    fn check(&self, obj: &T) -> CheckResult {
        self.iter()
            .flat_map(|f| f.check(obj))
            .collect::<Vec<_>>()
            .into()
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&self, obj: &mut T, u: &mut Unstructured<'static>) {
        for f in self.iter() {
            f.mutate(obj, u)
        }
    }
}

/// Convenience macro for creating a collection of `Constraint`s of different types.
/// The resulting value also implements `Constraint`.
#[macro_export]
macro_rules! constraints {
    ( $( $fact:expr ,)+ ) => {{
        let mut cs: $crate::ConstraintVec<_> = Vec::new();
        $(
            cs.push(Box::new($fact));
        )+
        Box::new(cs)
    }};
}
