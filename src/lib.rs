//! A trait for highly composable constraints ("facts") which can be used both
//! to verify data and to generate arbitrary data satisfying those constraints.
//!
//! This crate is mainly intended for use in writing tests, and in particular for generating meaningful fixture data. By defining composable, reusable constraints, they can be mixed and matched to handle the specific use cases of your tests. By defining what you need from a fixture rather than simply writing the fixture you need, the hope is that you save yourself duplicated effort over time.
//!
//! ## Example
//!
//! The following example defines a simple struct `S` with two fields, and a simple
//! Fact (constraint) about `S` which says that `S::x` must always equal `1`.
//! This Fact, like all Facts, can be used both to verify that an instance of `S`
//! meets the constraint, or to generate new instances of `S` which meet the constraint.
//!
//! ```
//! use contrafact::{Factual, facts::{eq, lens}};
//! use arbitrary::{Arbitrary, Unstructured};
//!
//! #[derive(Debug, Clone, PartialEq, Arbitrary)]
//! struct S {
//!     x: u32,
//!     y: u32,
//! }
//!
//! let mut fact = lens1("S::x", |s: &mut S| &mut s.x, eq(1));
//!
//! assert!(fact.clone().check(&S {x: 1, y: 333}).is_ok());
//! assert!(fact.clone().check(&S {x: 2, y: 333}).is_err());
//!
//! // NB: don't actually construct a Generator this way! See the docs for [[`Generator`]].
//! let mut g = contrafact::utils::random_generator();
//! let a = fact.build(&mut g);
//! assert_eq!(a.x, 1);
//! ```
//!
//! ## Things to know
//!
//! The above example composes together existing Facts provided by this
//! crate. You can also define your own facts by hand by implementing the `Fact`
//! trait. *TODO: example of this.*
//!
//! `contrafact` leans heavily on the [`arbitrary`](https://docs.rs/arbitrary/1.0.0/arbitrary/) crate for
//! generating arbitrary data. Get to know this library, because you will need to implement `Arbitrary` for any
//! type you wish to write a [`Fact`](crate::Fact) about.
//!
//! Facts can be used to check if a constraint is matched via [`Fact::check()`] or [`check_seq`],
//! and also to build new values via [`Fact::build`] and [`build_seq`]. Building values requires
//! the use of `arbitrary::Unstructured`.
//!
//! Facts can also be stateful, such that the constraint changes while checking or building a sequence. *TODO: example of stateful fact.*
//!
//! Facts can be easily "horizontally" composed together through the [`facts!`] macro, which
//! boxes each Fact and lumps them together as trait objects, applying each fact one after the other.
//!
//! Facts can be "vertically" composed together through the [`lens`] and [`prism`]
//! combinators, which allow you to lift a Fact about one type into a Fact about another type.
//!
//! See the Functions documentation for more examples and detailed instructions
//! about each Fact defined by this crate.

#![warn(missing_docs)]

mod check;
mod error;
mod fact;
/// Some built-in implementations of some useful facts
pub mod facts;
mod generator;
pub use facts::*;

#[cfg(feature = "utils")]
pub mod utils;

pub use arbitrary;

pub use check::Check;
pub use error::*;
pub use fact::{stateful, stateless, Fact, Fact2, State, StatelessFact, Target};
pub use generator::*;

pub use either;

/// The `brute` fact should only make this many attempts
pub(crate) const BRUTE_ITERATION_LIMIT: usize = 1000;

/// When running `Fact::satisfy`, repeat mutate+check this many times, in case
/// repetition helps ease into the constraint.
pub(crate) const SATISFY_ATTEMPTS: usize = 100;

/// Convenience macro for creating a collection of [`Fact`](crate::Fact)s
/// of different types.
/// The Facts will be composed into a nested series of [`AndFact`] which causes
/// all facts to be applied in sequence. The collection of Facts is also a Fact.
///
/// ```
/// use contrafact::*;
///
/// let eq1 = eq(1);
/// let not2 = not(eq(2));
/// let mut fact = facts![eq1, not2];
/// assert!(fact.check(&1).is_ok());
/// ```
#[macro_export]
macro_rules! facts {

    ( $fact:expr $(,)?) => { $fact };

    ( $fact_0:expr, $fact_1:expr $( , $fact_n:expr )* $(,)? ) => {{
        facts![
            $crate::facts::and($fact_0, $fact_1),
            $( $fact_n , )*
        ]
    }};
}
