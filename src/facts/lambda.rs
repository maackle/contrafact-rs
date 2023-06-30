use std::marker::PhantomData;

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
///     lambda(2, move |g, s, mut v| {
///         g.set(&mut v, s, "value is not geometrically increasing by 2")?;
///         *s *= 2;
///         Ok(v)
///     }),
/// );
///
/// let list = fact.clone().build(&mut g);
/// fact.check(&list).unwrap();
/// assert_eq!(list, vec![2, 4, 8, 16]);
/// ```
pub fn lambda<'a, T, S, F>(state: S, f: F) -> LambdaFact<'a, T, S, F>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
    F: Clone + Send + Sync + FnMut(&mut Generator, &mut S, T) -> Mutation<T>,
{
    LambdaFact {
        state,
        fun: f,
        _phantom: PhantomData,
    }
}

#[derive(Clone)]
pub struct LambdaFact<'a, T, S, F>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
    F: Clone + Send + Sync + FnMut(&mut Generator, &mut S, T) -> Mutation<T>,
{
    state: S,
    fun: F,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T, S, F> Fact<'a, T> for LambdaFact<'a, T, S, F>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
    F: Clone + Send + Sync + FnMut(&mut Generator, &mut S, T) -> Mutation<T>,
{
    fn mutate(&mut self, g: &mut Generator<'a>, obj: T) -> Mutation<T> {
        (self.fun)(g, &mut self.state, obj)
    }
}

#[test]
fn test_lambda_fact() {
    use crate::facts::*;
    let mut g = utils::random_generator();

    let fact = vec_of_length(
        4,
        lambda(2, move |g, s, mut v| {
            g.set(&mut v, s, "value is not geometrically increasing by 2")?;
            *s *= 2;
            Ok(v)
        }),
    );

    let list = fact.clone().build(&mut g);
    fact.check(&list).unwrap();
    assert_eq!(list, vec![2, 4, 8, 16]);
}
