mod and;
mod brute;
mod consecutive_int;
mod constant;
mod eq;
mod in_range;
mod in_slice;
mod lens;
mod mapped;
mod not;
mod or;
mod prism;
mod same;
mod seq;

pub use consecutive_int::{consecutive_int, consecutive_int_};
pub use constant::{always, never};
pub use eq::{eq, eq_, ne, ne_};
pub use in_range::{in_range, in_range_};
pub use in_slice::{in_slice, in_slice_};
pub use not::{not, not_};
pub use or::or;
pub use same::{different, same};

pub use and::and;
pub use brute::brute;
pub use lens::{lens, LensFact};
pub use mapped::{mapped, mapped_fallible};
pub use prism::{prism, PrismFact};
pub use seq::{vec, vec_len, vec_of_length};

// pub(crate) use lambda::LambdaFact;

#[cfg(feature = "optics")]
mod optical;
#[cfg(feature = "optics")]
pub use optical::*;

pub(crate) use eq::EqOp;

use crate::factual::check_raw;
use crate::*;
use std::marker::PhantomData;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;

    #[test]
    fn test_eq() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let eq1 = vec(eq("must be 1", 1));

        let ones = eq1.clone().build(&mut g);
        eq1.clone().check(&ones).unwrap();

        assert!(ones.iter().all(|x| *x == 1));
    }

    #[test]
    fn test_or() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let eq1 = eq("must be 1", 1);
        let eq2 = eq("must be 2", 2);
        let either = or("can be 1 or 2", eq1, eq2);

        let ones = vec(either.clone()).build(&mut g);
        vec(either.clone()).check(&ones).unwrap();
        assert!(ones.iter().all(|x| *x == 1 || *x == 2));

        assert_eq!(either.check(&3).result().unwrap().unwrap_err().len(), 1);
    }

    #[test]
    fn test_not() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let eq1 = eq("must be 1", 1);
        let not1 = vec(not_(eq1));

        let nums = not1.clone().build(&mut g);
        not1.clone().check(&nums).unwrap();

        assert!(nums.iter().all(|x| *x != 1));
    }

    #[test]
    fn test_same() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        {
            let f = vec(same::<u8>());
            let nums = f.clone().build(&mut g);
            f.clone().check(&nums).unwrap();
            assert!(nums.iter().all(|(a, b)| a == b));
        }
        {
            let f = vec(different::<u8>());
            let nums = f.clone().build(&mut g);
            f.clone().check(&nums).unwrap();
            assert!(nums.iter().all(|(a, b)| a != b));
        }
    }

    #[test]
    fn test_in_range() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let positive1 = in_range("must be positive", 1..=i32::MAX);
        let positive2 = in_range("must be positive", 1..);
        let smallish = in_range("must be small in magnitude", -10..100);
        let over9000 = in_range("must be over 9000", 9001..);
        let under9000 = in_range("must be under 9000 (and no less than zero)", ..9000u32);

        let nonpositive1 = vec(not_(positive1));
        let nonpositive2 = vec(not_(positive2));

        let smallish_nums = smallish.clone().build(&mut g);
        let over9000_nums = over9000.clone().build(&mut g);
        let under9000_nums = under9000.clone().build(&mut g);
        let nonpositive1_nums = nonpositive1.clone().build(&mut g);
        let nonpositive2_nums = nonpositive2.clone().build(&mut g);

        dbg!(&under9000_nums);

        smallish.clone().check(&smallish_nums).unwrap();
        over9000.clone().check(&over9000_nums).unwrap();
        under9000.clone().check(&under9000_nums).unwrap();
        nonpositive1.clone().check(&nonpositive1_nums).unwrap();
        nonpositive2.clone().check(&nonpositive2_nums).unwrap();
        assert!(nonpositive1_nums.iter().all(|x| *x <= 0));
    }
}
