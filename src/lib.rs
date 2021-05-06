//! Composable constraints ("facts") for coercing data into a certain shape,
//! or for verifying the shape of existing data

#![warn(missing_docs)]

mod custom;
mod derived;
mod fact;
mod lens;
mod predicates;
mod prism;

pub use custom::custom;
pub use derived::{build_seq, check_seq, DerivedFact};
pub use fact::{Fact, FactBox, FactVec};
pub use lens::lens;
pub use prism::prism;

/// The low-level building blocks of constraints
// TODO: maybe put this in the same namespace as the rest.
pub mod predicate {
    pub use super::predicates::{always, consecutive_int, eq, in_iter, ne, never, or};
}

#[cfg(any(test, feature = "test"))]
pub static NOISE: once_cell::sync::Lazy<Vec<u8>> = once_cell::sync::Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.gen()).take(999999).collect()
});
