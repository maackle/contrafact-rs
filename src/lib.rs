//! Composable constraints ("facts") for coercing data into a certain shape,
//! or for verifying the shape of existing data

#![warn(missing_docs)]

mod constraint;
mod fact;

use arbitrary::{Arbitrary, Unstructured};
pub use constraint::Constraints;
pub use fact::{Fact, FactSet};

/// Re-export of predicates with Constraint impls
pub mod predicate {
    pub use ::predicates::prelude::predicate::{eq, in_hash, in_iter};
}

/// Check that all of the constraints of all Facts are satisfied for this sequence.
pub fn check_seq<O>(seq: &mut [O], mut facts: FactSet<O>) {
    for obj in seq {
        for f in facts.0.iter_mut() {
            f.constraints()
                .checks
                .into_iter()
                .for_each(|check| check(obj))
        }
    }
}

/// Build a sequence from scratch such that all Facts are satisfied.
pub fn build_seq<O>(u: &mut Unstructured<'static>, num: usize, mut facts: FactSet<O>) -> Vec<O>
where
    O: Arbitrary<'static>,
{
    let mut seq = Vec::new();
    for _i in 0..num {
        let mut obj = O::arbitrary(u).unwrap();
        for f in facts.0.iter_mut() {
            for mutate in f.constraints().mutations.into_iter() {
                mutate(&mut obj, u)
            }
        }
        seq.push(obj);
    }
    return seq;
}
