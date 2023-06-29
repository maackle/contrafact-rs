//! Lift a fact about an item in a sequence into a Fact about the entire sequence.
//!
//! When checking or mutating this `seq` fact, the inner Fact will have `advance()`
//! called after each item. If the overall mutation fails due to a combination
//! of internally inconsistent facts, then the facts must be "rolled back" for the next
//! `satisfy()` attempt.

use std::marker::PhantomData;

use crate::*;

/// Lifts a Fact about an item in a sequence into a fact about the whole sequence.
///
/// ```
/// use contrafact::*;
/// use arbitrary::*;
///
/// todo!()
/// ```
//
// TODO: can rewrite this in terms of PrismFact for DRYness
pub fn seq<'a, T, F, S>(label: S, inner_fact: F) -> SeqFact<'a, T, F>
where
    T: Bounds<'a> + Clone,
    S: ToString,
    F: Fact<'a, T>,
{
    SeqFact::new(label.to_string(), inner_fact)
}

pub fn seq_<'a, T, F>(inner_fact: F) -> SeqFact<'a, T, F>
where
    T: Bounds<'a> + Clone,
    F: Fact<'a, T>,
{
    SeqFact::new("___", inner_fact)
}

pub fn sized_seq<'a, T, F>(len: usize, inner_fact: F) -> FactsRef<'a, Vec<T>>
where
    T: Bounds<'a> + Clone + 'a,
    F: Fact<'a, T> + 'a,
{
    facts![
        LenFact::new(len),
        SeqFact::new(format!("seq len {}", len), inner_fact)
    ]
}

/// A fact which uses a seq to apply another fact. Use [`seq()`] to construct.
#[derive(Clone)]
pub struct SeqFact<'a, T, F>
where
    T: Bounds<'a>,
    F: Fact<'a, T>,
{
    label: String,

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
    pub fn new<L>(label: L, inner_fact: F) -> Self
    where
        T: Bounds<'a>,
        F: Fact<'a, T>,
        L: ToString,
    {
        Self {
            label: label.to_string(),
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
    fn mutate(&self, obj: Vec<T>, g: &mut Generator<'a>) -> Mutation<Vec<T>> {
        tracing::trace!("");
        obj.into_iter()
            .map(|o| self.inner_fact.mutate(o, g))
            .collect::<Result<Vec<_>, _>>()
    }

    #[tracing::instrument(fields(fact = "seq"), skip(self))]
    fn advance(&mut self, obj: &Vec<T>) {
        tracing::trace!("seq");
        obj.iter().for_each(|o| self.inner_fact.advance(o));
    }
}

/// A fact which uses a seq to apply another fact. Use [`seq()`] to construct.
#[derive(Clone)]
pub struct LenFact<'a, T>
where
    T: Bounds<'a>,
{
    len: usize,
    __phantom: PhantomData<&'a T>,
}

impl<'a, T> LenFact<'a, T>
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

impl<'a, T> Fact<'a, Vec<T>> for LenFact<'a, T>
where
    T: Bounds<'a>,
{
    #[tracing::instrument(fields(fact = "len"), skip(self, g))]
    fn mutate(&self, mut obj: Vec<T>, g: &mut Generator<'a>) -> Mutation<Vec<T>> {
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

    #[tracing::instrument(fields(fact = "len"), skip(self))]
    fn advance(&mut self, obj: &Vec<T>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let ff = || {
            facts![
                brute("len must be >= 3", |v: &Vec<_>| v.len() >= 3),
                seq("S::x", eq("must be 1", 1)),
            ]
        };
        let mut f = ff();
        let ones = f.build(&mut g);
        f.check(&ones).unwrap();

        assert!(ones.len() >= 3);
        assert!(ones.iter().all(|s| *s == 1));
    }

    #[test]
    fn test_len() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let ones: Vec<u8> = LenFact::new(5).build(&mut g);
        LenFact::new(5).check(&ones).unwrap();

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
}
