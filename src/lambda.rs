use std::{fmt::Debug, marker::PhantomData, sync::Arc};

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
///     lambda("geometric series", 2, move |g, s, mut v| {
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
pub fn lambda<'a, S, T>(
    label: impl ToString,
    state: S,
    f: impl 'a + Send + Sync + Fn(&mut Generator<'a>, &mut S, T) -> Mutation<T>,
) -> Lambda<'a, S, T>
where
    S: State,
    T: Target<'a>,
{
    Lambda {
        label: label.to_string(),
        state,
        fun: Arc::new(f),
        _phantom: PhantomData,
    }
}

/// Create a lambda with unit state
pub fn lambda_unit<'a, T>(
    label: impl ToString,
    f: impl 'a + Send + Sync + Fn(&mut Generator<'a>, T) -> Mutation<T>,
) -> Lambda<'a, (), T>
where
    T: Target<'a>,
{
    lambda(label, (), move |g, (), t| f(g, t))
}

pub type LambdaFn<'a, S, T> =
    Arc<dyn 'a + Send + Sync + Fn(&mut Generator<'a>, &mut S, T) -> Mutation<T>>;

#[derive(Clone)]
pub struct Lambda<'a, S, T>
where
    S: State,
    T: Target<'a>,
{
    state: S,
    fun: LambdaFn<'a, S, T>,
    label: String,
    _phantom: PhantomData<&'a T>,
}

/// A Lambda with unit state
pub type LambdaUnit<'a, T> = Lambda<'a, (), T>;

impl<'a, S, T> std::fmt::Debug for Lambda<'a, S, T>
where
    S: State + Debug,
    T: Target<'a>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lambda")
            .field("label", &self.label)
            .field("state", &self.state)
            .finish()
    }
}

impl<'a, S, T> Fact<'a, T> for Lambda<'a, S, T>
where
    S: State + Debug,
    T: Target<'a>,
{
    fn mutate(&mut self, g: &mut Generator<'a>, t: T) -> Mutation<T> {
        (self.fun)(g, &mut self.state, t)
    }

    fn label(&self) -> String {
        self.label.clone()
    }

    fn labeled(mut self, label: impl ToString) -> Self {
        self.label = label.to_string();
        self
    }
}

#[test]
fn test_lambda_fact() {
    use crate::facts::*;
    let mut g = utils::random_generator();

    let geom = |k, s| {
        lambda("geom", s, move |g, s, mut v| {
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
