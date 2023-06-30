mod and;
mod brute;
mod consecutive_int;
mod constant;
mod eq;
mod in_range;
mod in_slice;
mod lens;
mod not;
mod or;
mod prism;
mod same;
mod seq;

pub use consecutive_int::{consecutive_int, consecutive_int_};
pub use constant::{always, never};
pub use eq::{eq, ne};
pub use in_range::in_range;
pub use in_slice::{in_slice, in_slice_};
pub use not::not;
pub use or::or;
pub use same::{different, same};

pub use and::and;
pub use brute::brute;
pub use lens::{lens1, lens2};
pub use prism::prism;
pub use seq::{vec, vec_len, vec_of_length};

// Optical facts are experimental and currently not supported
// #[cfg(feature = "optics")]
// mod optical;
// #[cfg(feature = "optics")]
// pub use optical::*;

use crate::*;
