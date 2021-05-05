//! Composable constraints ("facts") for coercing data into a certain shape,
//! or for verifying the shape of existing data

#![warn(missing_docs)]

mod constraint;
mod custom;
mod fact;
mod lens;
mod predicates;
mod prism;

pub use constraint::{Constraint, ConstraintBox, ConstraintVec};
pub use custom::{custom, CustomConstraint};
pub use fact::{build_seq, check_seq, Fact};
pub use lens::{lens, LensConstraint};
pub use prism::{prism, PrismConstraint};

/// The low-level building blocks of constraints
// TODO: maybe put this in the same namespace as the rest.
pub mod predicate {
    pub use super::predicates::{always, eq, in_iter, ne, never, or};
}

#[cfg(any(test, feature = "test"))]
pub static NOISE: once_cell::sync::Lazy<Vec<u8>> = once_cell::sync::Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.gen()).take(999999).collect()
});
