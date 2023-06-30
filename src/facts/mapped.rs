use crate::{factual::Bounds, *};

/// A version of [`mapped`] whose closure returns a Result
pub fn mapped_fallible<'a, T, F, O, S>(reason: impl ToString, f: F) -> Fact<'a, (), T>
where
    T: Bounds<'a>,
    O: Factual<'a, T>,
    F: 'a + Send + Sync + Fn(&T) -> ContrafactResult<O>,
{
    let reason = reason.to_string();
    stateless(move |g, obj| {
        f(&obj)?
            .mutate(g, obj)
            .map_check_err(|err| format!("mapped({}) > {}", reason, err))
    })
}

/// A fact which is defined based on the data to which it is applied. It maps
/// the data into a fact to be applied.
///
/// This can be useful for "piecewise" functions, where the
/// constraint is fundamentally different depending on the shape of the data,
/// or when wanting to set some subset of data to match some other subset of
/// data, without caring what the value actually is, and without having to
/// explicitly construct the value.
///
/// **NOTE**: since the returned Facts are generated brand-new on-the-fly,
/// these Facts must be stateless. State changes cannot be carried over to
/// subsequent calls when running over a sequence.
/// (TODO: add `StatelessFact` trait to give type-level protection here.)
///
/// ```
/// use contrafact::*;
///
/// // This contrived fact reads:
/// //   "if the number is greater than 9000,
/// //    ensure that it's also divisible by 9,
/// //    and otherwise, ensure that it's divisible by 10"
/// let mut fact = mapped("reason", |n: &u32| {
///     if *n > 9000 {
///         facts![ brute("divisible by 9", |n| *n % 9 == 0) ]
///     } else {
///         facts![ brute("divisible by 10", |n| *n % 10 == 0) ]
///     }
/// });
///
/// assert!(fact.clone().check(&50).is_ok());
/// assert!(fact.clone().check(&99).is_err());
/// assert!(fact.clone().check(&9009).is_ok());
/// assert!(fact.clone().check(&9010).is_err());
/// ```
pub fn mapped<'a, T, F, O>(reason: impl ToString, f: F) -> Fact<'a, (), T>
where
    T: Bounds<'a>,
    O: Factual<'a, T>,
    F: 'a + Send + Sync + Fn(&T) -> O,
{
    let reason = reason.to_string();
    stateless(move |g, obj| {
        f(&obj)
            .mutate(g, obj)
            .map_check_err(|err| format!("mapped({}) > {}", reason, err))
    })
}

#[test]
fn test_mapped_fact() {
    use crate::facts::*;

    type T = (u8, u8);

    let numbers = vec![(1, 11), (2, 22), (3, 33), (4, 44)];

    // This fact says:
    // if the first element of the tuple is even,
    //     then the second element must be divisible by 3;
    // and if the first element is odd,
    //     then the second element must be divisible by 4.
    let divisibility_fact = || {
        mapped("reason", |t: &T| {
            lens(
                "T.1",
                |(_, n)| n,
                if t.0 % 2 == 0 {
                    brute("divisible by 3", |n: &u8| n % 3 == 0)
                } else {
                    brute("divisible by 4", |n: &u8| n % 4 == 0)
                },
            )
        })
    };

    // assert that there was a failure
    vec(divisibility_fact())
        .check(&numbers)
        .result()
        .unwrap()
        .unwrap_err();

    // TODO: return all errors in the seq, not just the first
    // assert_eq!(
    //     dbg!(vec(divisibility_fact())
    //         .check(&numbers)
    //         .result()
    //         .unwrap()
    //         .unwrap_err()),
    //     vec![
    //         "item 0: mapped(reason) > lens(T.1) > divisible by 4".to_string(),
    //         "item 1: mapped(reason) > lens(T.1) > divisible by 3".to_string(),
    //         "item 2: mapped(reason) > lens(T.1) > divisible by 4".to_string(),
    //         "item 3: mapped(reason) > lens(T.1) > divisible by 3".to_string(),
    //     ]
    // );

    let mut g = utils::random_generator();

    let composite_fact = || {
        vec(facts![
            lens("T.0", |(i, _)| i, consecutive_int("increasing", 0)),
            divisibility_fact(),
        ])
    };

    let built = composite_fact().build(&mut g);
    dbg!(&built);
    composite_fact().check(&built).unwrap();
}
