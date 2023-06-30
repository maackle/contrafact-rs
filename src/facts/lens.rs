use crate::*;

/// Lifts a Fact about a subset of some data into a Fact about the superset,
/// using a single function to specify a getter/setter pair.
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
pub fn lens1<'a, O, T, L>(
    label: impl ToString,
    accessor: L,
    inner_fact: impl Fact<'a, T>,
) -> impl Fact<'a, O>
where
    O: Target<'a>,
    T: Target<'a>,
    L: 'a + Clone + Send + Sync + Fn(&mut O) -> &mut T,
{
    let accessor2 = accessor.clone();
    let getter = move |mut o| accessor(&mut o).clone();
    let setter = move |mut o, t: T| {
        let r = accessor2(&mut o);
        *r = t;
        o
    };
    lens2(label, getter, setter, inner_fact).labeled("lens1")
}

/// Lifts a Fact about a subset of some data into a Fact about the superset, using
/// explicit getter and setter functions.
///
/// This is a more general version of [`lens1`]. This can be useful particularly
/// when the setter requires modifications other than replacing the item specified
/// by the getter, for instance if your data contains some kind of digest of the data
/// being focused on, then the digest must also be recomputed when the focus is modified.
pub fn lens2<'a, O, T>(
    label: impl ToString,
    getter: impl 'a + Clone + Send + Sync + Fn(O) -> T,
    setter: impl 'a + Clone + Send + Sync + Fn(O, T) -> O,
    inner_fact: impl Fact<'a, T>,
) -> impl Fact<'a, O>
where
    O: Target<'a>,
    T: Target<'a>,
{
    let label = label.to_string();
    lambda("lens", inner_fact, move |g, fact, o: O| {
        let t = getter(o.clone());
        let t = fact
            .mutate(g, t)
            .map_check_err(|err| format!("lens1({}) > {}", label, err))?;
        Ok(setter(o, t))
    })
}

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
