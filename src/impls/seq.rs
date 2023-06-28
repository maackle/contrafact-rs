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
    fn test() {
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
}
