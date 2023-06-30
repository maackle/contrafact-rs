use std::{marker::PhantomData, sync::Arc};

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
pub fn stateful<'a, S, T>(
    state: S,
    f: impl 'a + Send + Sync + Fn(&mut Generator<'a>, &mut S, T) -> Mutation<T>,
) -> Fact<'a, S, T>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
{
    Fact {
        state,
        fun: Arc::new(f),
        _phantom: PhantomData,
    }
}

/// Create a lambda with unit state
pub fn stateless<'a, T>(
    f: impl 'a + Send + Sync + Fn(&mut Generator<'a>, T) -> Mutation<T>,
) -> Fact<'a, (), T>
where
    T: Bounds<'a>,
{
    stateful((), move |g, (), obj| f(g, obj))
}

pub type Lambda<'a, S, T> =
    Arc<dyn 'a + Send + Sync + Fn(&mut Generator<'a>, &mut S, T) -> Mutation<T>>;

#[derive(Clone)]
pub struct Fact<'a, S, T>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
{
    state: S,
    fun: Lambda<'a, S, T>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, S, T> Factual<'a, T> for Fact<'a, S, T>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
{
    fn mutate(&mut self, g: &mut Generator<'a>, obj: T) -> Mutation<T> {
        (self.fun)(g, &mut self.state, obj)
    }
}

#[test]
fn test_lambda_fact() {
    use crate::facts::*;
    let mut g = utils::random_generator();

    let geom = |k, s| {
        stateful(s, move |g, s, mut v| {
            g.set(
                &mut v,
                s,
                format!("value is not geometrically increasing by {k} starting from {s}"),
            )?;
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
