use std::{marker::PhantomData, sync::Arc};

use crate::*;

/// Lifts a Fact about a subset of some data into a Fact about the superset.
///
/// In other words, if type `O` contains a `T`, and you have a `Fact<'a, T>`,
/// `LensFact` lets you lift that fact into a `Fact<'a, O>`.
///
/// The `lens` closure provides a mutable view into the subset of data.
/// There must be a way to specify a mutable reference to the subset of data.
/// If this is not always possible, consider using [`prism()`](crate::prism) instead.
///
/// This is a lazy way to provide a lens in the traditional optics sense.
/// We may consider using a true lens library for this in the future.
///
/// ```
/// use contrafact::*;
/// use arbitrary::*;
///
/// #[derive(Debug, Clone, PartialEq, Arbitrary)]
/// struct S {
///     x: u32,
///     y: u32,
/// }
///
/// let mut fact = lens1("S::x", |s: &mut S| &mut s.x, eq(1));
///
/// assert!(fact.clone().check(&S {x: 1, y: 333}).is_ok());
/// assert!(fact.clone().check(&S {x: 2, y: 333}).is_err());
///
/// let mut g = utils::random_generator();
/// let a = fact.build(&mut g);
/// assert_eq!(a.x, 1);
/// ```
//
// TODO: can rewrite this in terms of PrismFact for DRYness
pub fn lens1<'a, O, T, L, S>(
    label: impl ToString,
    accessor: L,
    inner_fact: Fact<'a, S, T>,
) -> Fact<'a, Fact<'a, S, T>, O>
where
    O: Target<'a>,
    T: Target<'a>,
    S: State,
    L: 'a + Clone + Send + Sync + Fn(&mut O) -> &mut T,
{
    let accessor2 = accessor.clone();
    let getter = move |mut o| accessor(&mut o).clone();
    let setter = move |mut o, t: T| {
        let r = accessor2(&mut o);
        *r = t;
        o
    };
    lens2(label, getter, setter, inner_fact).label("lens1")
}

pub fn lens2<'a, O, T, S>(
    label: impl ToString,
    getter: impl 'a + Clone + Send + Sync + Fn(O) -> T,
    setter: impl 'a + Clone + Send + Sync + Fn(O, T) -> O,
    inner_fact: Fact<'a, S, T>,
) -> Fact<'a, Fact<'a, S, T>, O>
where
    O: Target<'a>,
    T: Target<'a>,
    S: State,
{
    let label = label.to_string();
    stateful("lens", inner_fact, move |g, fact, obj: O| {
        let t = getter(obj.clone());
        let t = fact
            .mutate(g, t)
            .map_check_err(|err| format!("lens1({}) > {}", label, err))?;
        Ok(setter(obj, t))
    })
}

// impl<'a, O, T, F> Factual<'a, O> for LensFact<'a, O, T, F>
// where
//     T: Bounds<'a>,
//     O: Bounds<'a> + Clone,
//     F: Factual<'a, T>,
// {
//     #[tracing::instrument(fields(fact = "lens"), skip(self, g))]
//     fn mutate(&mut self, g: &mut Generator<'a>, obj: O) -> Mutation<O> {
//         let t = (self.getter)(obj.clone());
//         let t = self
//             .inner_fact
//             .mutate(g, t)
//             .map_check_err(|err| format!("lens1({}) > {}", self.label, err))?;
//         Ok((self.setter)(obj, t))
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::*;
    use arbitrary::*;

    #[derive(Debug, Clone, PartialEq, Arbitrary)]
    struct S {
        x: u32,
        y: u32,
    }

    #[test]
    fn test() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let f = || vec(lens1("S::x", |s: &mut S| &mut s.x, eq(1)));
        let ones = f().build(&mut g);
        f().check(&ones).unwrap();

        assert!(ones.iter().all(|s| s.x == 1));
    }
}
