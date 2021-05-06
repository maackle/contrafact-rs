use arbitrary::*;

/// The trait bounds for the subject of a Fact
pub trait Bounds: std::fmt::Debug + PartialEq + Arbitrary<'static> + Clone {}
impl<T> Bounds for T where T: std::fmt::Debug + PartialEq + Arbitrary<'static> + Clone {}

/// Type alias for a boxed Fact
pub type BoxFact<'a, T> = Box<dyn 'a + Fact<T>>;

/// Type alias for a Vec of boxed Facts
pub type Facts<'a, T> = Vec<BoxFact<'a, T>>;

/// The result of a check operation, which contains an error message for every
/// constraint which was not met
// TODO: add ability to abort, so that further checks will not occur
#[derive(derive_more::From, derive_more::IntoIterator)]
#[must_use = "CheckResult should be used with either `.unwrap()` or `.ok()`"]
pub struct CheckResult {
    errors: Vec<String>,
}

impl CheckResult {
    /// Map over each error string
    pub fn map<F>(self, f: F) -> Self
    where
        F: FnMut(String) -> String,
    {
        if let Err(errs) = self.ok() {
            errs.into_iter().map(f).collect()
        } else {
            vec![]
        }
        .into()
    }

    /// Panic if there are any errors, and display those errors
    pub fn unwrap(self) {
        if !self.errors.is_empty() {
            let msg = if self.errors.len() == 1 {
                format!("Check failed: {}", self.errors[0])
            } else {
                format!("Check failed: {:#?}", self.errors)
            };
            panic!(msg);
        }
    }

    /// Convert to a Result: No errors => Ok
    pub fn ok(self) -> std::result::Result<(), Vec<String>> {
        if self.errors.is_empty() {
            std::result::Result::Ok(())
        } else {
            std::result::Result::Err(self.errors)
        }
    }

    /// Create an Ok result.
    pub fn pass() -> Self {
        Self {
            errors: Vec::with_capacity(0),
        }
    }

    /// Create an Ok result.
    pub fn fail(errors: Vec<String>) -> Self {
        Self { errors }
    }
}

/// A declarative representation of a constraint on some data, which can be
/// used to both make an assertion (check) or to mold some aribtrary existing
/// data into a shape which passes that same assertion (mutate)
pub trait Fact<T>
where
    T: Bounds,
{
    /// Assert that the constraint is satisfied (panic if not).
    fn check(&mut self, obj: &T) -> CheckResult;

    /// Mutate a value such that it satisfies the constraint.
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>);
}

impl<T, F> Fact<T> for Box<F>
where
    T: Bounds,
    F: Fact<T> + ?Sized,
{
    #[tracing::instrument(skip(self))]
    fn check(&mut self, obj: &T) -> CheckResult {
        tracing::trace!("check");
        (*self).as_mut().check(obj)
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>) {
        (*self).as_mut().mutate(obj, u);
    }
}

impl<T, F> Fact<T> for &mut [F]
where
    T: Bounds,
    F: Fact<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&mut self, obj: &T) -> CheckResult {
        self.iter_mut()
            .flat_map(|f| f.check(obj))
            .collect::<Vec<_>>()
            .into()
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>) {
        for f in self.iter_mut() {
            f.mutate(obj, u)
        }
    }
}

impl<T, F> Fact<T> for Vec<F>
where
    T: Bounds,
    F: Fact<T>,
{
    #[tracing::instrument(skip(self))]
    fn check(&mut self, obj: &T) -> CheckResult {
        self.iter_mut()
            .flat_map(|f| f.check(obj))
            .collect::<Vec<_>>()
            .into()
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&mut self, obj: &mut T, u: &mut Unstructured<'static>) {
        for f in self.iter_mut() {
            f.mutate(obj, u)
        }
    }
}
