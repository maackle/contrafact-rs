//! Lift a fact about an item in a sequence into a Fact about the entire sequence.
//!
//! When checking or mutating this `seq` fact, the inner Fact will have `advance()`
//! called after each item. If the overall mutation fails due to a combination
//! of internally inconsistent facts, then the facts must be "rolled back" for the next
//! `satisfy()` attempt.

use crate::*;

use super::and;

/// Lifts a Fact about an item in a Vec into a fact about the whole Vec.
///
///
/// ```
/// use contrafact::{*, facts::*};
///
/// let mut g = utils::random_generator();
///
/// // `consecutive_int`
/// let fact = facts::vec(facts::eq(1));
/// let list = fact.clone().satisfy(&mut g, vec![0; 5]).unwrap();
/// assert_eq!(list, vec![1, 1, 1, 1, 1]);
/// ```
///
/// When using a Fact which modifies its state,
///
/// ```
/// use contrafact::{*, facts::*};
///
/// let mut g = utils::random_generator();
///
/// // `consecutive_int`
/// let fact = vec(consecutive_int_(0));
/// let list = fact.clone().satisfy(&mut g, vec![0; 5]).unwrap();
/// assert_eq!(list, vec![0, 1, 2, 3, 4]);
/// ```
pub fn vec<'a, T>(inner_fact: impl Fact<'a, T>) -> impl Fact<'a, Vec<T>>
where
    T: Target<'a> + Clone,
{
    lambda("vec", inner_fact, |g, f, obj: Vec<T>| {
        obj.into_iter()
            .enumerate()
            .map(|(i, o)| {
                f.mutate(g, o)
                    .map_check_err(|e| format!("seq[{}]: {}", i, e))
            })
            .collect::<Result<Vec<_>, _>>()
    })
}

/// Checks that a Vec is of a given length
pub fn vec_len<'a, T>(len: usize) -> LambdaUnit<'a, Vec<T>>
where
    T: Target<'a> + Clone + 'a,
{
    lambda_unit("vec_len", move |g, mut obj: Vec<T>| {
        if obj.len() > len {
            g.fail(format!(
                "vec should be of length {} but is actually of length {}",
                len,
                obj.len()
            ))?;
            obj = obj[0..len].to_vec();
        }
        while obj.len() < len {
            obj.push(g.arbitrary(|| {
                format!(
                    "vec should be of length {} but is actually of length {}",
                    len,
                    obj.len()
                )
            })?)
        }
        Ok(obj)
    })
}

/// Combines a LenFact with a VecFact to ensure that the vector is of a given length
pub fn vec_of_length<'a, T>(len: usize, inner_fact: impl Fact<'a, T>) -> impl Fact<'a, Vec<T>>
where
    T: Target<'a> + 'a,
{
    and(vec_len(len), vec(inner_fact))
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    };

    use crate::facts::*;
    use arbitrary::*;

    #[derive(Debug, Clone, PartialEq, Arbitrary)]
    struct S {
        x: u32,
        y: u32,
    }

    #[test]
    fn test_seq() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let f = facts![
            brute("len must be >= 3", |v: &Vec<_>| v.len() >= 3),
            vec(eq(1)),
        ];
        let ones = f.clone().build(&mut g);
        f.check(&ones).unwrap();

        assert!(ones.len() >= 3);
        assert!(ones.iter().all(|s| *s == 1));
    }

    #[test]
    fn test_len() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let ones: Vec<u8> = vec_len(5).build(&mut g);
        vec_len(5).check(&ones).unwrap();

        assert_eq!(ones.len(), 5);
    }

    #[test]
    fn test_sized_seq() {
        let mut g = utils::random_generator();

        let f = || vec_of_length(5, consecutive_int_(0));
        let count: Vec<u8> = f().build(&mut g);
        f().check(&count).unwrap();

        assert_eq!(count, vec![0, 1, 2, 3, 4]);
    }

    /// Assert that even when satisfy() requires a fact to be run
    /// multiple times due to contradictory facts, if the constraint
    /// can be eventually satisfied, the facts still advance only
    /// when the constraint is met, and not during each failed attempt.
    #[test]
    fn test_mutation_replay() {
        let mut g = utils::random_generator();

        let piecewise = move || {
            let count = Arc::new(AtomicU8::new(0));
            lambda("piecewise", (), move |g, (), mut obj| {
                let c = count.fetch_add(1, Ordering::SeqCst);
                if c < 3 {
                    g.set(&mut obj, &999, || "i'm being difficult, haha")?;
                }
                Ok(obj)
            })
        };

        // Assert that the consecutive_int fact does not advance when there
        // is a failure for the facts to agree
        {
            let f = vec_of_length(10, facts!(consecutive_int_(0), piecewise()));
            let items = f.build(&mut g);
            assert_eq!(items, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        }

        // Assert that piecewise() messes everything up during the
        // first 3 mutations, and cooperates afterwards
        {
            let mut f = facts!(eq(0), piecewise());
            for _ in 0..3 {
                let val = f.mutate(&mut g, 0).unwrap();
                assert!(f.clone().check(&val).is_err());
            }
            let val = f.mutate(&mut g, 0).unwrap();
            f.check(&val).unwrap();
        }
    }
}
