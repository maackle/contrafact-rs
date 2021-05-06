//! Composable constraints ("facts") for coercing data into a certain shape,
//! or for verifying the shape of existing data

#![warn(missing_docs)]

mod conditional;
mod custom;
mod fact;
mod lens;
mod predicates;
mod prism;
mod seq;

pub use conditional::conditional;
pub use custom::custom;
pub use fact::{BoxFact, CheckResult, Fact, Facts};
pub use lens::lens;
pub use predicates::{
    always, consecutive_int, consecutive_int_, eq, eq_, in_iter, in_iter_, ne, ne_, never, or,
};
pub use prism::prism;
pub use seq::*;

#[cfg(any(test, feature = "test"))]
pub static NOISE: once_cell::sync::Lazy<Vec<u8>> = once_cell::sync::Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.gen()).take(999999).collect()
});
