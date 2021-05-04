//! Composable constraints ("facts") for coercing data into a certain shape,
//! or for verifying the shape of existing data

#![warn(missing_docs)]

mod constraint;
mod custom;
mod fact;
mod lens;
mod predicates;
mod stateful;

pub use constraint::{Constraint, ConstraintBox, ConstraintVec};
pub use fact::{build_seq, check_seq, Fact};
pub use lens::{lens, LensConstraint};
// pub use stateful::{stateful, StatefulFact};

pub mod predicate {
    pub use super::predicates::{eq, in_iter, ne, or};
}
