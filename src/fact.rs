use arbitrary::*;

use crate::*;

/// When running `Fact::satisfy`, repeat mutate+check this many times, in case
/// repetition helps ease into the constraint.
pub(crate) const SATISFY_ATTEMPTS: usize = 7;

// TODO: we can remove the Clone requirement if:
// - make `Mutate` track list of errors so that it can know if a mutation occurred.
// - make `mutate()` take a mut ref
// - make `check()` take a mut ref
// then `check()` can know if a mutation occurred
//
/// The trait bounds for the subject of a Fact
pub trait Bounds<'a>: std::fmt::Debug + Clone + PartialEq + Arbitrary<'a> {}
impl<'a, T> Bounds<'a> for T where T: std::fmt::Debug + Clone + PartialEq + Arbitrary<'a> {}

/// Type alias for a boxed Fact. Implements [`Fact`] itself.
pub type BoxFact<'a, T> = Box<dyn 'a + Fact<'a, T>>;

/// Type alias for a Vec of boxed Facts. Implements [`Fact`] itself.
pub type FactsRef<'a, T> = Vec<BoxFact<'a, T>>;

/// Type alias for a static Vec of boxed Facts. Implements [`Fact`] itself.
pub type Facts<T> = FactsRef<'static, T>;

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
        Check::from_mutation(self.mutate(obj.clone(), &mut g))
    }

    /// Apply a mutation which moves the obj closer to satisfying the overall
    /// constraint.
    fn mutate(&self, obj: T, g: &mut Generator<'a>) -> Mutation<T>;

    /// When checking or mutating a sequence of items, this gets called after
    /// each item to modify the state to get ready for the next item.
    fn advance(&mut self, obj: &T);

    /// Mutate a value such that it satisfies the constraint.
    /// If the constraint cannot be satisfied, panic.
    fn satisfy(&mut self, mut obj: T, g: &mut Generator<'a>) -> ContrafactResult<T> {
        let mut last_failure: Vec<String> = vec![];
        for _i in 0..SATISFY_ATTEMPTS {
            obj = self.mutate(obj, g).unwrap();
            if let Err(errs) = self.check(&obj).result()? {
                last_failure = errs;
            } else {
                return Ok(obj);
            }
        }
        panic!(
            "Could not satisfy a constraint even after {} attemps. Last check failure: {:?}",
            SATISFY_ATTEMPTS, last_failure
        );
    }

    /// Build a new value such that it satisfies the constraint
    fn build(&mut self, g: &mut Generator<'a>) -> ContrafactResult<T> {
        let obj = T::arbitrary(g).unwrap();
        self.satisfy(obj, g)
    }
}

impl<'a, T, F> Fact<'a, T> for Box<F>
where
    T: Bounds<'a>,
    F: Fact<'a, T> + ?Sized,
{
    #[tracing::instrument(skip(self, g))]
    fn mutate(&self, obj: T, g: &mut Generator<'a>) -> Mutation<T> {
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
        collect_checks(self, obj)
    }

    #[tracing::instrument(skip(self, g))]
    fn mutate(&self, mut obj: T, g: &mut Generator<'a>) -> Mutation<T> {
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
        collect_checks(self.as_slice(), obj)
    }

    #[tracing::instrument(skip(self, g))]
    fn mutate(&self, mut obj: T, g: &mut Generator<'a>) -> Mutation<T> {
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

fn collect_checks<'a, T, F>(facts: &[F], obj: &T) -> Check
where
    T: Bounds<'a>,
    F: Fact<'a, T>,
{
    let checks = facts
        .iter()
        .enumerate()
        .map(|(i, f)| {
            Ok(f.check(obj)
                .failures()?
                .iter()
                .map(|e| format!("fact {}: {}", i, e))
                .collect())
        })
        .collect::<ContrafactResult<Vec<Vec<Failure>>>>()
        .map(|fs| fs.into_iter().flatten().collect());
    Check::from_result(checks)
}
