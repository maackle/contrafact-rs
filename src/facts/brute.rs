use std::sync::Arc;

use crate::{fact::Bounds, Fact, BRUTE_ITERATION_LIMIT};

use crate::{lambda_unit, ContrafactResult, Generator, Mutation};

use super::lambda::LambdaFact;

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
/// fn div_by(n: usize) -> impl Fact<'static, usize> {
///     facts![brute(format!("Is divisible by {}", n), move |x| x % n == 0)]
/// }
///
/// let mut g = utils::random_generator();
/// assert!(div_by(3).build(&mut g) % 3 == 0);
/// ```
pub fn brute<'a, T, F>(reason: impl ToString, f: F) -> LambdaFact<'a, (), T>
where
    T: Bounds<'a>,
    F: 'a + Send + Sync + Fn(&T) -> bool,
{
    let reason = reason.to_string();
    brute_labeled(move |v| Ok(f(v).then_some(()).ok_or_else(|| reason.clone())))
}

/// A version of [`brute`] which allows the closure to return the reason for failure
pub fn brute_labeled<'a, T, F>(f: F) -> LambdaFact<'a, (), T>
where
    T: Bounds<'a>,
    F: 'a + Send + Sync + Fn(&T) -> ContrafactResult<BruteResult>,
{
    lambda_unit(move |g, mut obj| {
        let mut last_reason = "".to_string();
        for _ in 0..=BRUTE_ITERATION_LIMIT {
            if let Err(reason) = f(&obj)? {
                obj = g.arbitrary(&reason)?;
                last_reason = reason;
            } else {
                return Ok(obj);
            }
        }

        panic!(
            "Exceeded iteration limit of {} while attempting to meet a BruteFact. Last failure reason: {}",
            BRUTE_ITERATION_LIMIT, last_reason
        );
    })
}

type BruteResult = Result<(), String>;
