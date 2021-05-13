use std::sync::Arc;

use arbitrary::Unstructured;

use crate::{check_fallible, fact::Bounds, Check, Fact, Facts};

/// A version of `conditional` whose closure returns a Result
pub fn conditional_fallible<'a, T, F, S>(reason: S, f: F) -> ConditionalFact<'a, T>
where
    S: ToString,
    T: Bounds,
    F: 'static + Fn(&T) -> crate::Result<Facts<'a, T>>,
{
    ConditionalFact::new(reason.to_string(), f)
}

/// A constraint where the data to be constrained determines which constraint
/// to apply.
///
/// This can be useful for "piecewise" functions, where the
/// constraint is fundamentally different depending on the shape of the data,
/// or when wanting to set some subset of data to match some other subset of
/// data, without caring what the value actually is, and without having to
/// explicitly construct the value.
pub fn conditional<T, F, S>(reason: S, f: F) -> ConditionalFact<'static, T>
where
    S: ToString,
    T: Bounds,
    F: 'static + Fn(&T) -> Facts<'static, T>,
{
    ConditionalFact::new(reason.to_string(), move |x| Ok(f(x)))
}

#[derive(Clone)]
pub struct ConditionalFact<'a, T> {
    reason: String,
    f: Arc<dyn 'a + Fn(&T) -> crate::Result<Facts<'a, T>>>,
}

impl<'a, T> Fact<T> for ConditionalFact<'a, T>
where
    T: Bounds,
{
    fn check(&self, t: &T) -> Check {
        check_fallible! {{
            Ok((self.f)(t)?
            .check(t)
            .map(|e| format!("conditional({}) > {}", self.reason, e)))
        }}
    }

    fn mutate(&self, t: &mut T, u: &mut Unstructured<'static>) {
        (self.f)(t).expect("TODO: fallible mutation").mutate(t, u)
    }

    fn advance(&mut self, _: &T) {}
}

impl<'a, T> ConditionalFact<'a, T> {
    pub(crate) fn new<F: 'a + Fn(&T) -> crate::Result<Facts<'a, T>>>(reason: String, f: F) -> Self {
        Self {
            reason,
            f: Arc::new(f),
        }
    }
}

#[test]
fn test_conditional_fact() {
    use crate::*;
    type T = (u8, u8);

    let numbers = vec![(1, 11), (2, 22), (3, 33), (4, 44)];

    // This fact says:
    // if the first element of the tuple is even,
    //     then the second element must be divisible by 3;
    // and if the first element is odd,
    //     then the second element must be divisible by 4.
    let divisibility_fact = || {
        conditional("reason", |t: &T| {
            facts![lens(
                "T.1",
                |(_, n)| n,
                if t.0 % 2 == 0 {
                    custom("divisible by 3", |n: &u8| n % 3 == 0)
                } else {
                    custom("divisible by 4", |n: &u8| n % 4 == 0)
                }
            ),]
        })
    };
    assert_eq!(
        dbg!(check_seq(numbers.as_slice(), divisibility_fact())
            .ok()
            .unwrap_err()),
        vec![
            "item 0: conditional(reason) > lens(T.1) > divisible by 4".to_string(),
            "item 1: conditional(reason) > lens(T.1) > divisible by 3".to_string(),
            "item 2: conditional(reason) > lens(T.1) > divisible by 4".to_string(),
            "item 3: conditional(reason) > lens(T.1) > divisible by 3".to_string(),
        ]
    );

    let mut u = Unstructured::new(&NOISE);

    let composite_fact = || {
        facts![
            lens("T.0", |(i, _)| i, consecutive_int("increasing", 0)),
            divisibility_fact(),
        ]
    };

    let built = build_seq(&mut u, 12, composite_fact());
    dbg!(&built);
    check_seq(built.as_slice(), composite_fact()).unwrap();
}
