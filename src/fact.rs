use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use arbitrary::Arbitrary;

use crate::*;

/// Create a fact from a bare function which specifies the mutation.
/// Can be quicker to experiment with ideas this way than to have to directly implement
/// the [`Fact`] trait
///
/// ```
/// use contrafact::*;
/// let mut g = utils::random_generator();
///
/// let mut fact = vec_of_length(
///     4,
///     stateful("geometric series", 2, move |g, s, mut v| {
///         g.set(&mut v, s, || "value is not geometrically increasing by 2")?;
///         *s *= 2;
///         Ok(v)
///     }),
/// );
///
/// let list = fact.clone().build(&mut g);
/// fact.check(&list).unwrap();
/// assert_eq!(list, vec![2, 4, 8, 16]);
/// ```
pub fn stateful<'a, S, T>(
    label: impl ToString,
    state: S,
    f: impl 'a + Send + Sync + Fn(&mut Generator<'a>, &mut S, T) -> Mutation<T>,
) -> Fact<'a, S, T>
where
    S: Clone + Send + Sync,
    T: Target<'a>,
{
    Fact {
        label: label.to_string(),
        state,
        fun: Arc::new(f),
        _phantom: PhantomData,
    }
}

/// Create a lambda with unit state
pub fn stateless<'a, T>(
    label: impl ToString,
    f: impl 'a + Send + Sync + Fn(&mut Generator<'a>, T) -> Mutation<T>,
) -> Fact<'a, (), T>
where
    T: Target<'a>,
{
    stateful(label, (), move |g, (), obj| f(g, obj))
}

pub type Lambda<'a, S, T> =
    Arc<dyn 'a + Send + Sync + Fn(&mut Generator<'a>, &mut S, T) -> Mutation<T>>;

#[derive(Clone)]
pub struct Fact<'a, S, T>
where
    S: Clone + Send + Sync,
    T: Target<'a>,
{
    state: S,
    fun: Lambda<'a, S, T>,
    label: String,
    _phantom: PhantomData<&'a T>,
}

/// Two facts about the same target with different states
pub type Fact2<'a, A, B, T> = (Fact<'a, A, T>, Fact<'a, B, T>);

impl<'a, S, T> std::fmt::Debug for Fact<'a, S, T>
where
    S: Clone + Send + Sync + Debug,
    T: Target<'a>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Fact")
            .field("label", &self.label)
            .field("state", &self.state)
            .finish()
    }
}

impl<'a, S, T> Fact<'a, S, T>
where
    S: Clone + Send + Sync + Debug,
    T: Target<'a>,
{
    /// Change the label
    pub fn label(mut self, label: impl ToString) -> Self {
        self.label = label.to_string();
        self
    }

    /// Assert that the constraint is satisfied for given data.
    ///
    /// If the mutation function is written properly, we get a check for free
    /// by using a special Generator which fails upon mutation. If this is for
    /// some reason unreasonable, a check function can be written by hand, but
    /// care must be taken to make sure it perfectly lines up with the mutation function.
    #[tracing::instrument(fields(fact_impl = "Fact"), skip(self))]
    pub fn check(mut self, obj: &T) -> Check {
        let mut g = Generator::checker();
        Check::from_mutation(self.mutate(&mut g, obj.clone()))
    }

    /// Apply a mutation which moves the obj closer to satisfying the overall
    /// constraint.
    // #[tracing::instrument(skip(self, g))]
    pub fn mutate(&mut self, g: &mut Generator<'a>, obj: T) -> Mutation<T> {
        (self.fun)(g, &mut self.state, obj)
    }

    /// Make this many attempts to satisfy a constraint before giving up and panicking.
    ///
    /// If you are combining highly contentious facts together and relying on randomness
    /// to find a solution, this limit may need to be higher. In general, you should try
    /// to write facts that don't interfere with each other so that the constraint can be
    /// met on the first attempt, or perhaps the second or third. If necessary, this can
    /// be raised to lean more on random search.
    pub fn satisfy_attempts(&self) -> usize {
        SATISFY_ATTEMPTS
    }

    /// Mutate a value such that it satisfies the constraint.
    /// If the constraint cannot be satisfied, panic.
    pub fn satisfy(&mut self, g: &mut Generator<'a>, obj: T) -> ContrafactResult<T> {
        tracing::trace!("satisfy");
        let mut last_failure: Vec<String> = vec![];
        let mut next = obj.clone();
        for _i in 0..self.satisfy_attempts() {
            let mut m = self.clone();
            let mut c = self.clone();
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
    pub fn build_fallible(mut self, g: &mut Generator<'a>) -> ContrafactResult<T> {
        let obj = T::arbitrary(g).unwrap();
        self.satisfy(g, obj)
    }

    /// Build a new value such that it satisfies the constraint, panicking on error
    #[tracing::instrument(fields(fact_impl = "Fact"), skip(self, g))]
    pub fn build(self, g: &mut Generator<'a>) -> T {
        self.build_fallible(g).unwrap()
    }
}

/// A Fact with unit state
pub type StatelessFact<'a, T> = Fact<'a, (), T>;

#[test]
fn test_lambda_fact() {
    use crate::facts::*;
    let mut g = utils::random_generator();

    let geom = |k, s| {
        stateful("geom", s, move |g, s, mut v| {
            g.set(&mut v, s, || {
                format!("value is not geometrically increasing by {k} starting from {s}")
            })?;
            *s *= k;
            Ok(v)
        })
    };

    let fact = |k, s| vec_of_length(4, geom(k, s));

    {
        let list = fact(2, 2).build(&mut g);
        assert_eq!(list, vec![2, 4, 8, 16]);
        fact(2, 2).check(&list).unwrap();
    }

    {
        let list = fact(3, 4).build(&mut g);
        assert_eq!(list, vec![4, 12, 36, 108]);
        fact(3, 4).check(&list).unwrap();
    }
}
