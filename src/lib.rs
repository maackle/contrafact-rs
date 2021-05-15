//! Composable constraints ("facts") for coercing data into a certain shape,
//! or for verifying the shape of existing data.
//!
//! ## Motivation
//!
//! When testing,

#![warn(missing_docs)]

mod check;
mod fact;
mod impls;
mod satisfy;

pub use arbitrary;

pub use check::Check;
pub use fact::{BoxFact, Fact, Facts};
pub use satisfy::*;

pub use impls::primitives::{
    always, consecutive_int, consecutive_int_, eq, eq_, in_iter, in_iter_, ne, ne_, never, not,
    not_, or,
};

pub use impls::brute::{brute, brute_fallible, BruteFact};
pub use impls::lens::{lens, LensFact};
pub use impls::mapped::{mapped, mapped_fallible, MappedFact};
pub use impls::prism::{prism, PrismFact};

/// The Result type returnable when using [`check_fallible!`]
pub type Result<T> = anyhow::Result<T>;

pub(crate) const BRUTE_ITERATION_LIMIT: usize = 100;

#[cfg(any(test, feature = "test"))]
pub static NOISE: once_cell::sync::Lazy<Vec<u8>> = once_cell::sync::Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.gen()).take(999999).collect()
});
