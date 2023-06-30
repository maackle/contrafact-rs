use arbitrary::*;
use either::Either;

use crate::*;

/// The trait bounds for the target of a Fact
pub trait Target<'a>:
    'a + std::fmt::Debug + Clone + Send + Sync + PartialEq + Arbitrary<'a>
{
}
impl<'a, T> Target<'a> for T where
    T: 'a + std::fmt::Debug + Clone + Send + Sync + PartialEq + Arbitrary<'a>
{
}

/// The trait bounds for the State of a Fact
pub trait State: std::fmt::Debug + Clone + Send + Sync {}
impl<T> State for T where T: std::fmt::Debug + Clone + Send + Sync {}

/// A declarative representation of a constraint on some data, which can be
/// used to both make an assertion (check) or to mold some arbitrary existing
/// data into a shape which passes that same assertion (mutate)
pub trait Fact<'a, T>: Send + Sync + Clone + std::fmt::Debug
where
    T: Target<'a>,
{
    /// Change the label
    fn labeled(self, label: impl ToString) -> Self;

    /// Get the label of this fact
    fn label(&self) -> String;

    /// Assert that the constraint is satisfied for given data.
    ///
    /// If the mutation function is written properly, we get a check for free
    /// by using a special Generator which fails upon mutation. If this is for
    /// some reason unreasonable, a check function can be written by hand, but
    /// care must be taken to make sure it perfectly lines up with the mutation function.
    #[tracing::instrument(fields(fact_impl = "Fact"), skip(self))]
    fn check(mut self, t: &T) -> Check {
        let mut g = Generator::checker();
        Check::from_mutation(self.mutate(&mut g, t.clone()))
    }

    /// Apply a mutation which moves the t closer to satisfying the overall
    /// constraint.
    // #[tracing::instrument(skip(self, g))]
    fn mutate(&mut self, g: &mut Generator<'a>, t: T) -> Mutation<T>;

    /// Make this many attempts to satisfy a constraint before giving up and panicking.
    ///
    /// If you are combining highly contentious facts together and relying on randomness
    /// to find a solution, this limit may need to be higher. In general, you should try
    /// to write facts that don't interfere with each other so that the constraint can be
    /// met on the first attempt, or perhaps the second or third. If necessary, this can
    /// be raised to lean more on random search.
    fn satisfy_attempts(&self) -> usize {
        SATISFY_ATTEMPTS
    }

    /// Mutate a value such that it satisfies the constraint.
    /// If the constraint cannot be satisfied, panic.
    #[tracing::instrument(fields(fact_impl = "Fact"), skip(self, g))]
    fn satisfy(&mut self, g: &mut Generator<'a>, t: T) -> ContrafactResult<T> {
        tracing::trace!("satisfy");
        let mut last_failure: Vec<String> = vec![];
        let mut next = t.clone();
        for _i in 0..self.satisfy_attempts() {
            let mut m = self.clone();
            next = m.mutate(g, next).unwrap();
            if let Err(errs) = self.clone().check(&next).result()? {
                last_failure = errs;
            } else {
                *self = m;
                return Ok(next);
            }
        }
        panic!(
            "Could not satisfy a constraint even after {} attempts. Last check failure: {:?}",
            SATISFY_ATTEMPTS, last_failure
        );
    }

    #[tracing::instrument(fields(fact_impl = "Fact"), skip(self, g))]
    /// Build a new value such that it satisfies the constraint
    fn build_fallible(mut self, g: &mut Generator<'a>) -> ContrafactResult<T> {
        let t = T::arbitrary(g).unwrap();
        self.satisfy(g, t)
    }

    /// Build a new value such that it satisfies the constraint, panicking on error
    #[tracing::instrument(fields(fact_impl = "Fact"), skip(self, g))]
    fn build(self, g: &mut Generator<'a>) -> T {
        self.build_fallible(g).unwrap()
    }
}

impl<'a, T, F1, F2> Fact<'a, T> for Either<F1, F2>
where
    T: Target<'a>,
    F1: Fact<'a, T> + ?Sized,
    F2: Fact<'a, T> + ?Sized,
{
    #[tracing::instrument(fields(fact_impl = "Either"), skip(self, g))]
    fn mutate(&mut self, g: &mut Generator<'a>, t: T) -> Mutation<T> {
        match self {
            Either::Left(f) => f.mutate(g, t),
            Either::Right(f) => f.mutate(g, t),
        }
    }

    fn label(&self) -> String {
        match self {
            Either::Left(f) => f.label(),
            Either::Right(f) => f.label(),
        }
    }

    fn labeled(self, label: impl ToString) -> Self {
        match self {
            Either::Left(f) => Either::Left(f.labeled(label)),
            Either::Right(f) => Either::Right(f.labeled(label)),
        }
    }
}

#[tracing::instrument(skip(facts))]
fn collect_checks<'a, T, F>(facts: Vec<F>, t: &T) -> Check
where
    T: Target<'a>,
    F: Fact<'a, T>,
{
    let checks = facts
        .into_iter()
        .enumerate()
        .map(|(i, f)| {
            Ok(f.check(t)
                .failures()?
                .iter()
                .map(|e| format!("fact {}: {}", i, e))
                .collect())
        })
        .collect::<ContrafactResult<Vec<Vec<Failure>>>>()
        .map(|fs| fs.into_iter().flatten().collect());
    Check::from_result(checks)
}
