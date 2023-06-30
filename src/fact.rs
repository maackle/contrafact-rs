use arbitrary::*;
use either::Either;

use crate::*;

// TODO: we can remove the Clone requirement if:
// - make `Mutate` track list of errors so that it can know if a mutation occurred.
// - make `mutate()` take a mut ref
// - make `check()` take a mut ref
// then `check()` can know if a mutation occurred
//
/// The trait bounds for the subject of a Fact
pub trait Bounds<'a>: std::fmt::Debug + Clone + Send + Sync + PartialEq + Arbitrary<'a> {}
impl<'a, T> Bounds<'a> for T where
    T: std::fmt::Debug + Clone + Send + Sync + PartialEq + Arbitrary<'a>
{
}

// /// Type alias for a boxed Fact. Implements [`Fact`] itself.
// pub type BoxFact<'a, T> = Box<dyn 'a + Fact<'a, T>>;

// pub trait Facts<T: Bounds<'static>>: Fact<'static, T> {}
// impl<T: Bounds<'static>, F: Facts<T>> Fact<'static, T> for F {}

/// A declarative representation of a constraint on some data, which can be
/// used to both make an assertion (check) or to mold some arbitrary existing
/// data into a shape which passes that same assertion (mutate)
pub trait Fact<'a, T>: Send + Sync + Clone
where
    T: Bounds<'a>,
{
    /// Assert that the constraint is satisfied for given data.
    ///
    /// If the mutation function is written properly, we get a check for free
    /// by using a special Generator which fails upon mutation. If this is for
    /// some reason unreasonable, a check function can be written by hand, but
    /// care must be taken to make sure it perfectly lines up with the mutation function.
    #[tracing::instrument(fields(fact_impl = "Fact"), skip(self))]
    fn check(mut self, obj: &T) -> Check {
        check_raw(&mut self, obj)
    }

    /// Apply a mutation which moves the obj closer to satisfying the overall
    /// constraint.
    // #[tracing::instrument(skip(self, g))]
    fn mutate(&mut self, g: &mut Generator<'a>, obj: T) -> Mutation<T>;

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
    fn satisfy(&mut self, g: &mut Generator<'a>, obj: T) -> ContrafactResult<T> {
        tracing::trace!("satisfy");
        let mut last_failure: Vec<String> = vec![];
        let mut next = obj.clone();
        for _i in 0..self.satisfy_attempts() {
            let mut m = self.clone();
            let mut c = self.clone();
            next = m.mutate(g, next).unwrap();
            if let Err(errs) = check_raw(&mut c, &next).result()? {
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
        let obj = T::arbitrary(g).unwrap();
        self.satisfy(g, obj)
    }

    /// Build a new value such that it satisfies the constraint, panicking on error
    #[tracing::instrument(fields(fact_impl = "Fact"), skip(self, g))]
    fn build(self, g: &mut Generator<'a>) -> T {
        self.build_fallible(g).unwrap()
    }
}

impl<'a, T, F1, F2> Fact<'a, T> for Either<F1, F2>
where
    T: Bounds<'a>,
    F1: Fact<'a, T> + ?Sized,
    F2: Fact<'a, T> + ?Sized,
{
    #[tracing::instrument(fields(fact_impl = "Either"), skip(self, g))]
    fn mutate(&mut self, g: &mut Generator<'a>, obj: T) -> Mutation<T> {
        match self {
            Either::Left(f) => f.mutate(g, obj),
            Either::Right(f) => f.mutate(g, obj),
        }
    }
}

#[tracing::instrument(skip(fact))]
pub(crate) fn check_raw<'a, T, F: Fact<'a, T>>(fact: &mut F, obj: &T) -> Check
where
    T: Bounds<'a> + ?Sized,
    F: Fact<'a, T> + ?Sized,
{
    let mut g = Generator::checker();
    Check::from_mutation(fact.mutate(&mut g, obj.clone()))
}

#[tracing::instrument(skip(facts))]
fn collect_checks<'a, T, F>(facts: Vec<F>, obj: &T) -> Check
where
    T: Bounds<'a>,
    F: Fact<'a, T>,
{
    let checks = facts
        .into_iter()
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
