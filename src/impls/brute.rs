use std::sync::Arc;

use arbitrary::Unstructured;

use crate::{check_fallible, fact::Bounds, Check, Fact, BRUTE_ITERATION_LIMIT};

/// A version of [`brute`] whose closure returns a Result
pub fn brute_fallible<'a, T, F, S>(reason: S, f: F) -> BruteFact<'a, T>
where
    S: ToString,
    T: Bounds<'a>,
    F: 'a + Fn(&T) -> crate::Result<bool>,
{
    BruteFact::<T>::new(reason.to_string(), f)
}

/// A constraint defined only by a predicate closure. Mutation occurs by brute
/// force, randomly trying values until one matches the constraint.
///
/// This is appropriate to use when the space of possible values is small, and
/// you can rely on randomness to eventually find a value that matches the
/// constraint through sheer brute force, e.g. when requiring a particular
/// enum variant.
///
/// **NOTE**: When doing mutation, this constraint can do no better than
/// brute force when finding data that satisfies the constraint. Therefore,
/// if the predicate is unlikely to return `true` given arbitrary data,
/// this constraint is a bad choice!
///
/// ALSO **NOTE**: It is usually best to place this constraint at the beginning
/// of a chain when doing mutation, because if the closure specifies a weak
/// constraint, the mutation may drastically alter the data, potentially undoing
/// constraints that were met by previous mutations.
///
/// There is a fixed iteration limit, beyond which this will panic.
///
/// ```
/// use arbitrary::Unstructured;
/// use contrafact::*;
///
/// fn div_by(n: usize) -> Facts<usize> {
///     facts![brute(format!("Is divisible by {}", n), move |x| x % n == 0)]
/// }
///
/// let mut u = Unstructured::new(&[0; 9999]);
/// assert!(div_by(3).build(&mut u) % 3 == 0);
/// ```
pub fn brute<'a, T, F, S>(reason: S, f: F) -> BruteFact<'a, T>
where
    S: ToString,
    T: Bounds<'a>,
    F: 'a + Fn(&T) -> bool,
{
    BruteFact::<T>::new(reason.to_string(), move |x| Ok(f(x)))
}

type BruteFn<'a, T> = Arc<dyn 'a + (Fn(&T) -> crate::Result<bool>)>;

/// A brute-force fact. Use [`brute()`] to construct.
#[derive(Clone)]
pub struct BruteFact<'a, T> {
    label: String,
    f: BruteFn<'a, T>,
}

impl<'a, T> Fact<'a, T> for BruteFact<'a, T>
where
    T: Bounds<'a>,
{
    fn check(&self, t: &T) -> Check {
        check_fallible!({ Ok(Check::check((self.f)(t)?, self.label.clone())) })
    }

    #[cfg(feature = "mutate-inplace")]
    fn mutate(&self, t: &mut T, u: &mut Unstructured<'a>) {
        for _ in 0..BRUTE_ITERATION_LIMIT {
            if (self.f)(&t).expect("Mutation failed.") {
                return;
            }
            t = T::arbitrary(u).unwrap();
        }

        panic!(
            "Exceeded iteration limit of {} while attempting to meet a BruteFact. Context: {}",
            BRUTE_ITERATION_LIMIT, self.reason
        );
    }

    #[cfg(feature = "mutate-owned")]
    fn mutate(&self, mut t: T, u: &mut Unstructured<'a>) -> T {
        for _ in 0..BRUTE_ITERATION_LIMIT {
            if (self.f)(&t).expect("Mutation failed.") {
                return t;
            }
            t = T::arbitrary(u).unwrap();
        }

        panic!(
            "Exceeded iteration limit of {} while attempting to meet a BruteFact. Context: {}",
            BRUTE_ITERATION_LIMIT, self.label
        );
    }

    fn advance(&mut self, _: &T) {}
}

impl<'a, T> BruteFact<'a, T> {
    pub(crate) fn new<F: 'a + Fn(&T) -> crate::Result<bool>>(reason: String, f: F) -> Self {
        Self {
            label: reason,
            f: Arc::new(f),
        }
    }
}
