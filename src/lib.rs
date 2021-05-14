//! Composable constraints ("facts") for coercing data into a certain shape,
//! or for verifying the shape of existing data

#![warn(missing_docs)]

mod brute;
mod check;
mod dependent;
mod fact;
mod lens;
mod primitives;
mod prism;
mod seq;

pub use arbitrary;

pub use brute::{brute, brute_fallible};
pub use check::Check;
pub use dependent::{dependent, dependent_fallible};
pub use fact::{BoxFact, Fact, Facts};
pub use lens::lens;
pub use primitives::{
    always, consecutive_int, consecutive_int_, eq, eq_, in_iter, in_iter_, ne, ne_, never, not,
    not_, or,
};
pub use prism::prism;
pub use seq::*;

/// The Result type returnable when using `check_fallible!`
pub type Result<T> = anyhow::Result<T>;

#[cfg(any(test, feature = "test"))]
pub static NOISE: once_cell::sync::Lazy<Vec<u8>> = once_cell::sync::Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.gen()).take(999999).collect()
});
