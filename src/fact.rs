use arbitrary::*;

use crate::{check::CheckError, Check};

/// When running `Fact::satisfy`, repeat mutate+check this many times, in case
/// repetition helps ease into the constraint.
pub(crate) const SATISFY_ATTEMPTS: usize = 7;

/// The trait bounds for the subject of a Fact
pub trait Bounds<'a>: std::fmt::Debug + Clone + PartialEq + Arbitrary<'a> {}
impl<'a, T> Bounds<'a> for T where T: std::fmt::Debug + Clone + PartialEq + Arbitrary<'a> {}

/// Type alias for a boxed Fact. Implements [`Fact`] itself.
pub type BoxFact<'a, T> = Box<dyn 'a + Fact<'a, T>>;

/// Type alias for a Vec of boxed Facts. Implements [`Fact`] itself.
pub type FactsRef<'a, T> = Vec<BoxFact<'a, T>>;

/// Type alias for a static Vec of boxed Facts. Implements [`Fact`] itself.
pub type Facts<T> = FactsRef<'static, T>;

/// Mutation errors must give String reasons for mutation, which can be used to
/// specify the error when used for a Check
pub type GenResult<T> = Result<T, CheckError>;

/// Type used to generate new values and error messages
#[must_use = "Be sure to use Generator::value even if you're assigning a constant value, to provide an error message when running check()"]
#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Generator<'a> {
    arb: Option<Unstructured<'a>>,
}

impl<'a> From<Unstructured<'a>> for Generator<'a> {
    fn from(arb: Unstructured<'a>) -> Self {
        Self { arb: Some(arb) }
    }
}

impl<'a> From<&'a [u8]> for Generator<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        arbitrary::Unstructured::new(bytes).into()
    }
}

impl<'a> Generator<'a> {
    pub fn checker() -> Self {
        Self { arb: None }
    }

    pub fn fail(&self, err: impl ToString) -> GenResult<()> {
        if self.arb.is_none() {
            Err(err.to_string())
        } else {
            Ok(())
        }
    }

    pub fn value<T>(&mut self, val: T, err: impl ToString) -> GenResult<T> {
        if self.arb.is_none() {
            Err(err.to_string())
        } else {
            Ok(val)
        }
    }

    pub fn arbitrary<T: Arbitrary<'a>>(&mut self, err: impl ToString) -> GenResult<T> {
        self.with(err, |u| u.arbitrary())
    }

    pub fn choose<T: Arbitrary<'a>>(
        &mut self,
        choices: &'a [T],
        err: impl ToString,
    ) -> GenResult<&T> {
        self.with(err, |u| u.choose(choices))
    }

    pub fn with<T>(
        &mut self,
        err: impl ToString,
        f: impl FnOnce(&mut Unstructured<'a>) -> Result<T, arbitrary::Error>,
    ) -> GenResult<T> {
        if let Some(mut arb) = self.arb.as_mut() {
            f(&mut arb).map_err(|e| format!("Could not generate data: {}", e))
        } else {
            Err(err.to_string())
        }
    }
}

/// A declarative representation of a constraint on some data, which can be
/// used to both make an assertion (check) or to mold some arbitrary existing
/// data into a shape which passes that same assertion (mutate)
pub trait Fact<'a, T>
where
    T: Bounds<'a>,
{
    /// Assert that the constraint is satisfied for given data
    fn check(&self, obj: &T) -> Check {
        let mut g = Generator::checker();
        Check::from_result(self.mutate(obj.clone(), &mut g))
    }

    /// Apply a mutation which moves the obj closer to satisfying the overall
    /// constraint.
    fn mutate(&self, obj: T, g: &mut Generator<'a>) -> GenResult<T>;

    /// When checking or mutating a sequence of items, this gets called after
    /// each item to modify the state to get ready for the next item.
    fn advance(&mut self, obj: &T);

    /// Mutate a value such that it satisfies the constraint.
    /// If the constraint cannot be satisfied, panic.
    fn satisfy(&mut self, mut obj: T, g: &mut Generator<'a>) -> T {
        let mut last_failure: Vec<String> = vec![];
        for _i in 0..SATISFY_ATTEMPTS {
            obj = self
                .mutate(obj, g)
                .expect("Ran out of Unstructured data. Try again with more Unstructured bytes.");
            if let Err(errs) = self.check(&obj).result() {
                last_failure = errs;
            } else {
                return obj;
            }
        }
        panic!(
            "Could not satisfy a constraint even after {} attemps. Last check failure: {:?}",
            SATISFY_ATTEMPTS, last_failure
        );
    }

    /// Build a new value such that it satisfies the constraint
    fn build(&mut self, g: &mut Generator<'a>) -> T {
        let obj = g
            .arbitrary("Ran out of Unstructured data. Try again with more Unstructured bytes.")
            .unwrap();
        self.satisfy(obj, g)
    }
}

impl<'a, T, F> Fact<'a, T> for Box<F>
where
    T: Bounds<'a>,
    F: Fact<'a, T> + ?Sized,
{
    #[tracing::instrument(skip(self, g))]
    fn mutate(&self, obj: T, g: &mut Generator<'a>) -> GenResult<T> {
        (*self).as_ref().mutate(obj, g)
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

    #[tracing::instrument(skip(self, g))]
    fn mutate(&self, mut obj: T, g: &mut Generator<'a>) -> GenResult<T> {
        for f in self.iter() {
            obj = f.mutate(obj, g)?;
        }
        Ok(obj)
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

    #[tracing::instrument(skip(self, g))]
    fn mutate(&self, mut obj: T, g: &mut Generator<'a>) -> GenResult<T> {
        for f in self.iter() {
            obj = f.mutate(obj, g)?;
        }
        Ok(obj)
    }

    #[tracing::instrument(skip(self))]
    fn advance(&mut self, obj: &T) {
        for f in self.iter_mut() {
            f.advance(obj)
        }
    }
}
