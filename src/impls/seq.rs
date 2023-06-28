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
    fn check(&mut self, seq: &Vec<T>) -> Check {
        let mut g = Generator::checker();

        for (i, obj) in seq.iter().enumerate() {
            let check = Check::from_mutation(self.inner_fact.mutate(obj.clone(), &mut g));
            if let Ok(Ok(())) = check.clone().result() {
                self.inner_fact.advance(&obj);
            } else {
                return check.map(|e| format!("seq({})[{}]: {}", self.label, i, e));
            }
        }
        Check::pass()
    }

    fn satisfy(&mut self, seq: Vec<T>, g: &mut Generator<'a>) -> ContrafactResult<Vec<T>> {
        let satisfy_attempts = self.satisfy_attempts();
        let mut last_failure: Vec<String> = vec![];
        let mut next_seq = Vec::with_capacity(seq.len());
        'item: for (i, obj) in seq.iter().enumerate() {
            let mut next = obj.clone();
            for _a in 0..satisfy_attempts {
                next = self.inner_fact.mutate(next, g).unwrap();
                if let Err(errs) = self
                    .inner_fact
                    .check(&next)
                    .map(|e| format!("seq({})[{}]: {}", self.label, i, e))
                    .result()?
                {
                    last_failure = errs
                } else {
                    self.inner_fact.advance(&obj);
                    next_seq.push(next);
                    continue 'item;
                }
            }
            panic!(
                "Could not satisfy a constraint even after {} attempts. Last check failure: {:?}",
                satisfy_attempts, last_failure
            );
        }
        Ok(next_seq)
    }

    fn mutate(&self, _: Vec<T>, _: &mut Generator<'a>) -> Mutation<Vec<T>> {
        unimplemented!("satisfy() was implemented directly")
    }

    fn advance(&mut self, _: &Vec<T>) {
        unimplemented!("satisfy() was implemented directly")
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
