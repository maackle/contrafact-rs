//! Composable constraints ("facts") for coercing data into a certain shape,
//! or for verifying the shape of existing data

#![warn(missing_docs)]

mod facts;

use arbitrary::{Arbitrary, Unstructured};
pub use facts::{Fact, FactGen, FactSet, LensFact};
use predicates::contrafact::Bounds;

/// Re-export of predicates with Constraint impls
pub mod fact {
    pub use crate::facts::{lens, predicate};
    pub use ::predicates::prelude::predicate::{eq, in_hash, in_iter};
}

/// Check that all of the constraints of all Facts are satisfied for this sequence.
pub fn check_seq<O, F>(seq: &mut [O], mut fact: F)
where
    F: Fact<O>,
    O: Bounds,
{
    for obj in seq {
        fact.check(obj)
    }
}

/// Build a sequence from scratch such that all Facts are satisfied.
pub fn build_seq<O, F>(u: &mut Unstructured<'static>, num: usize, mut fact: F) -> Vec<O>
where
    O: Arbitrary<'static> + Bounds,
    F: Fact<O>,
{
    let mut seq = Vec::new();
    for _i in 0..num {
        let mut obj = O::arbitrary(u).unwrap();
        fact.mutate(&mut obj, u);
        seq.push(obj);
    }
    return seq;
}
