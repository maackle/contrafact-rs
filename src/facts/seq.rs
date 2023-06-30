//! Lift a fact about an item in a sequence into a Fact about the entire sequence.
//!
//! When checking or mutating this `seq` fact, the inner Fact will have `advance()`
//! called after each item. If the overall mutation fails due to a combination
//! of internally inconsistent facts, then the facts must be "rolled back" for the next
//! `satisfy()` attempt.

use std::marker::PhantomData;

use crate::*;

use super::and;

/// Lifts a Fact about an item in a sequence into a fact about the whole sequence.
///
///
/// ```
/// use contrafact::{*, facts::*};
///
/// let mut g = utils::random_generator();
///
/// // `consecutive_int`
/// let fact = facts::seq(facts::eq_(1));
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
/// let fact = seq(consecutive_int_(0));
/// let list = fact.clone().satisfy(&mut g, vec![0; 5]).unwrap();
/// assert_eq!(list, vec![0, 1, 2, 3, 4]);
/// ```
///
//
// TODO: can rewrite this in terms of PrismFact for DRYness
pub fn seq<'a, T, F>(inner_fact: F) -> SeqFact<'a, T, F>
where
    T: Bounds<'a> + Clone,
    F: Fact<'a, T>,
{
    SeqFact::new(inner_fact)
}

/// Checks that a Vec is of a given length
pub fn seq_len<'a, T>(len: usize) -> SeqLenFact<'a, T>
where
    T: Bounds<'a> + Clone + 'a,
{
    SeqLenFact::new(len)
}

/// Combines a LenFact with a SeqFact to ensure that the sequence is of a given length
pub fn sized_seq<'a, T, F>(len: usize, inner_fact: F) -> impl Fact<'a, Vec<T>>
where
    T: Bounds<'a> + Clone + 'a,
    F: Fact<'a, T> + 'a,
{
    and(seq_len(len), seq(inner_fact))
}

/// A fact which uses a seq to apply another fact. Use [`seq()`] to construct.
#[derive(Clone)]
pub struct SeqFact<'a, T, F>
where
    T: Bounds<'a>,
    F: Fact<'a, T>,
{
    /// The inner_fact about the inner substructure
    inner_fact: F,

    __phantom: PhantomData<&'a T>,
}

impl<'a, T, F> SeqFact<'a, T, F>
where
    T: Bounds<'a>,
    F: Fact<'a, T>,
{
    /// Constructor. Supply a seq and an existing Fact to create a new Fact.
    pub fn new(inner_fact: F) -> Self
    where
        T: Bounds<'a>,
        F: Fact<'a, T>,
    {
        Self {
            inner_fact,
            __phantom: PhantomData,
        }
    }
}

impl<'a, T, F> Fact<'a, Vec<T>> for SeqFact<'a, T, F>
where
    T: Bounds<'a>,
    F: Fact<'a, T>,
{
    #[tracing::instrument(fields(fact = "seq"), skip(self, g))]
    fn mutate(&mut self, g: &mut Generator<'a>, obj: Vec<T>) -> Mutation<Vec<T>> {
        tracing::trace!("");
        obj.into_iter()
            .enumerate()
            .map(|(i, o)| {
                self.inner_fact
                    .mutate(g, o)
                    .map_check_err(|e| format!("seq[{}]: {}", i, e))
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

/// A fact which uses a seq to apply another fact. Use [`seq()`] to construct.
#[derive(Clone)]
pub struct SeqLenFact<'a, T>
where
    T: Bounds<'a>,
{
    len: usize,
    __phantom: PhantomData<&'a T>,
}

impl<'a, T> SeqLenFact<'a, T>
where
    T: Bounds<'a>,
{
    /// Constructor. Supply a seq and an existing Fact to create a new Fact.
    pub fn new(len: usize) -> Self
    where
        T: Bounds<'a>,
    {
        Self {
            len,
            __phantom: PhantomData,
        }
    }
}

impl<'a, T> Fact<'a, Vec<T>> for SeqLenFact<'a, T>
where
    T: Bounds<'a>,
{
    #[tracing::instrument(fields(fact = "len"), skip(self, g))]
    fn mutate(&mut self, g: &mut Generator<'a>, mut obj: Vec<T>) -> Mutation<Vec<T>> {
        tracing::trace!("");

        if obj.len() > self.len {
            g.fail("LenFact: vec was too long")?;
            obj = obj[0..self.len].to_vec();
        }
        while obj.len() < self.len {
            obj.push(g.arbitrary("LenFact: vec was too short")?)
        }
        Ok(obj)
    }
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
            seq(eq("must be 1", 1)),
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

        let ones: Vec<u8> = seq_len(5).build(&mut g);
        seq_len(5).check(&ones).unwrap();

        assert_eq!(ones.len(), 5);
    }

    #[test]
    fn test_sized_seq() {
        let mut g = utils::random_generator();

        let f = || sized_seq(5, consecutive_int_(0));
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
            lambda((), move |g, (), mut obj| {
                let c = count.fetch_add(1, Ordering::SeqCst);
                if c < 3 {
                    g.set(&mut obj, &999, "i'm being difficult, haha")?;
                }
                Ok(obj)
            })
        };

        // Assert that the consecutive_int fact does not advance when there
        // is a failure for the facts to agree
        {
            let f = sized_seq(10, facts!(consecutive_int_(0), piecewise()));
            let items = f.build(&mut g);
            assert_eq!(items, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        }

        // Assert that piecewise() messes everything up during the
        // first 3 mutations, and cooperates afterwards
        {
            let mut f = facts!(eq_(0), piecewise());
            for _ in 0..3 {
                let val = f.mutate(&mut g, 0).unwrap();
                assert!(f.clone().check(&val).is_err());
            }
            let val = f.mutate(&mut g, 0).unwrap();
            f.check(&val).unwrap();
        }
    }
}
