use std::{marker::PhantomData, sync::Arc};

use crate::*;

/// Lifts a Fact about some *optional* subset of data into a Fact about the
/// superset.
///
/// In other words, if type `O` contains a `Option<T>`, and you have a `Fact<T>`,
/// `PrismFact` lets you lift that fact into a `Fact<O>`.
///
/// The `prism` closure provides an optional mutable view into the subset.
/// If the prism returns None during any fact application, the fact will
/// effectively be skipped for this item: no check or mutation will be performed,
/// and the state will not advance.
///
/// A prism is like a lens, except that the target value may or may not exist.
/// It is typically used for enums, or any structure where data may or may not
/// be present.
///
/// ```
/// use contrafact::*;
/// use arbitrary::{Arbitrary, Unstructured};
///
/// #[derive(Debug, Clone, PartialEq, Arbitrary)]
/// enum E {
///     X(u32),
///     Y(u32),
/// }
///
/// impl E {
///     fn x(&mut self) -> Option<&mut u32> {
///         match self {
///             E::X(x) => Some(x),
///             _ => None,
///         }
///     }
///     fn y(&mut self) -> Option<&mut u32> {
///         match self {
///             E::Y(y) => Some(y),
///             _ => None,
///         }
///     }
/// }
///
/// let mut fact = prism("E::x", E::x, eq(1));
///
/// assert!(fact.clone().check(&E::X(1)).is_ok());
/// assert!(fact.clone().check(&E::X(2)).is_err());
/// assert!(fact.clone().check(&E::Y(99)).is_ok());
///
/// let mut g = utils::random_generator();
/// let e = fact.build(&mut g);
/// match e {
///     E::X(x) => assert_eq!(x, 1),
///     _ => (),  // Y is not defined by the prism, so it can take on any value.
/// };
/// ```
///
/// The `prism` closure is a rather lazy way to provide a prism in the
/// traditional optics sense. We may consider using a true lens library for
/// this in the future.
pub fn prism<'a, O, T, P, S>(
    label: impl ToString,
    prism: P,
    inner_fact: Fact<'a, S, T>,
) -> Fact<'a, Fact<'a, S, T>, O>
where
    O: Target<'a>,
    T: Target<'a>,
    P: 'a + Send + Sync + Fn(&mut O) -> Option<&mut T>,
    S: State,
{
    let label = label.to_string();
    stateful("prism", inner_fact, move |g, fact, mut obj| {
        if let Some(t) = prism(&mut obj) {
            *t = fact
                .mutate(g, t.clone())
                .map_check_err(|err| format!("prism({}) > {}", label, err))?;
        }
        Ok(obj)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;
    use arbitrary::*;

    #[derive(Debug, Clone, PartialEq, Arbitrary)]
    enum E {
        X(u32),
        Y(u32),
    }

    impl E {
        fn x(&mut self) -> Option<&mut u32> {
            match self {
                E::X(x) => Some(x),
                _ => None,
            }
        }
        fn y(&mut self) -> Option<&mut u32> {
            match self {
                E::Y(y) => Some(y),
                _ => None,
            }
        }
    }

    #[test]
    fn stateless() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let f = || {
            facts::vec(facts![
                prism("E::x", E::x, facts::eq(1)),
                prism("E::y", E::y, facts::eq(2)),
            ])
        };

        let items = f().build(&mut g);
        f().check(&items).unwrap();

        assert!(items.iter().all(|e| match e {
            E::X(x) => *x == 1,
            E::Y(y) => *y == 2,
        }))
    }

    #[test]
    fn stateful() {
        use itertools::*;
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let f = || {
            facts::vec(facts![
                prism(
                    "E::x",
                    E::x,
                    facts::consecutive_int("must be increasing", 0),
                ),
                prism(
                    "E::y",
                    E::y,
                    facts::consecutive_int("must be increasing", 0),
                ),
            ])
        };

        let items = f().build(&mut g);
        f().check(&items).unwrap();

        // Assert that each variant of E is independently increasing
        let (xs, ys): (Vec<_>, Vec<_>) = items.into_iter().partition_map(|e| match e {
            E::X(x) => Either::Left(x),
            E::Y(y) => Either::Right(y),
        });
        facts::vec(crate::facts![facts::consecutive_int_(0u32)])
            .check(&xs)
            .unwrap();
        facts::vec(crate::facts![facts::consecutive_int_(0u32)])
            .check(&ys)
            .unwrap();
    }
}
