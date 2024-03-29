//! Generators create new arbitrary data to attempt to satisfy constraints, and associate
//! contextual messages with those instances of data generation.
//!
//! All Facts are written in terms of a mutation function which makes use of a Generator.
//! If a mutation results in no change to the data, that implies that all constraints are satisfied.
//! If a mutation does need to change data, that implies that a constraint is not met.
//! We can make use of this implication to be able to both detect data which does not satisfy a constraint,
//! as well as to mutate the data to better satisfy a constraint.
//!
//! Every Generator operation has an associated error message.
//! When running a Fact::check, the generator throws an error any time it is used, which signals that a constraint
//! was not met. When running Fact::mutate, no error is thrown, and new data is produced instead.
//! All Facts must be written with this dual use in mind.

use arbitrary::{Arbitrary, Unstructured};

use crate::error::*;
use arbitrary::unstructured::Int;
use std::ops::RangeInclusive;

/// Generators are used to generate new values and error messages.
///
/// For mutation logic which actually generates new data, error messages are produced instead of data during a Check.
/// In some cases, `Generator::fail` must be used when attempting to mutate data using existing values not generated by Generator.
#[must_use = "Be sure to use Generator::fail even if you're not generating new values, to provide an error message when running check()"]
#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Generator<'a> {
    #[deref]
    #[deref_mut]
    arb: Unstructured<'a>,

    check: bool,
}

impl<'a> From<Unstructured<'a>> for Generator<'a> {
    fn from(arb: Unstructured<'a>) -> Self {
        assert!(!arb.is_empty());
        Self { arb, check: false }
    }
}

impl<'a> From<&'a [u8]> for Generator<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        arbitrary::Unstructured::new(bytes).into()
    }
}

impl<'a> Generator<'a> {
    pub(crate) fn checker() -> Self {
        Self {
            arb: arbitrary::Unstructured::new(&[]),
            check: true,
        }
    }

    /// When running a Check, fail immediately with this error.
    /// This should be used in cases where a mutation occurs using some known value, rather than
    /// generating a value from the Generator itself.
    pub fn fail(&self, err: impl ToString) -> Mutation<()> {
        if self.check {
            Err(MutationError::Check(err.to_string()))
        } else {
            Ok(())
        }
    }

    /// When running a Check, fail immediately with this error if the existing value doesn't match.
    /// During mutation, set the value so that it does match.
    pub fn set<T: PartialEq + Clone, S: ToString>(
        &self,
        source: &mut T,
        target: &T,
        err: impl FnOnce() -> S,
    ) -> Mutation<()> {
        if source != target {
            if self.check {
                return Err(MutationError::Check(err().to_string()));
            } else {
                *source = target.clone();
            }
        }
        Ok(())
    }

    /// Generate arbitrary data in mutation mode, or produce an error in check mode
    pub fn arbitrary<T: Arbitrary<'a>, S: ToString>(
        &mut self,
        err: impl FnOnce() -> S,
    ) -> Mutation<T> {
        self.with(err, |u| u.arbitrary())
    }

    /// Choose between specified items in mutation mode, or produce an error in check mode.
    pub fn choose<T: Arbitrary<'a>, S: ToString>(
        &mut self,
        choices: &'a [T],
        err: impl FnOnce() -> S,
    ) -> Mutation<&T> {
        if choices.is_empty() {
            return Err(MutationError::User("Empty choices".to_string())).into();
        }
        if choices.len() == 1 {
            return Ok(&choices[0]).into();
        }
        if !self.check && self.arb.is_empty() {
            return Err(MutationError::User("Ran out of entropy".to_string())).into();
        }
        self.with(err, |u| u.choose(choices))
    }

    /// Exposes `int_in_range` from underlying `Unstructured` in mutation mode,
    /// or produce an error in check mode.
    /// Note that even though arbitrary says NOT to use this for calculating the
    /// size of a collection to build, that's exactly what I will be doing with
    /// this, because I'm not sure exactly what the contrafact story is for
    /// size hints (which is what arbitrary would be using instead).
    pub fn int_in_range<T, S>(
        &mut self,
        range: RangeInclusive<T>,
        err: impl FnOnce() -> S,
    ) -> Mutation<T>
    where
        T: Arbitrary<'a> + PartialOrd + Copy + Int,
        S: ToString,
    {
        if range.start() > range.end() {
            return Err(MutationError::User("Invalid range".to_string())).into();
        } else if range.start() == range.end() {
            return Ok(*range.start()).into();
        }
        if !self.check && self.arb.is_empty() {
            return Err(MutationError::User("Ran out of entropy".to_string())).into();
        }
        self.with(err, |u| u.int_in_range(range))
    }

    /// Call the specified Arbitrary function in mutation mode, or produce an error in check mode.
    pub fn with<T, S: ToString>(
        &mut self,
        err: impl FnOnce() -> S,
        f: impl FnOnce(&mut Unstructured<'a>) -> Result<T, arbitrary::Error>,
    ) -> Mutation<T> {
        if self.check {
            Err(MutationError::Check(err().to_string())).into()
        } else {
            f(&mut self.arb).map_err(Into::into)
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::MutationError;
    use rand::prelude::SliceRandom;
    use rand::SeedableRng;

    /// Test that int_in_range won't accept an invalid range.
    #[test]
    pub fn test_generator_int_in_range_invalid_range() {
        let mut gen = crate::generator::Generator::from(&[0, 1, 2, 3, 4, 5][..]);
        assert_eq!(
            gen.int_in_range(5..=4, || "error"),
            Err(MutationError::User("Invalid range".to_string()))
        );
    }

    /// Test that int_in_range will accept a valid range of a single option.
    #[test]
    pub fn test_generator_int_in_range_valid_range_single_option() {
        let mut gen = crate::generator::Generator::from(&[0][..]);
        for _ in 0..10 {
            assert_eq!(gen.int_in_range(5..=5, || "error").unwrap(), 5);
        }
        // The generator has not had any entropy consumed by this point.
        assert_eq!(gen.len(), 1);
    }

    /// Test that int_in_range will accept a valid range of multiple options.
    #[test]
    pub fn test_generator_int_in_range_valid_range_multiple_options() {
        let mut gen = crate::generator::Generator::from(&[0, 1, 2, 3, 4, 5][..]);
        for i in 0..6 {
            assert_eq!(gen.int_in_range(0..=3, || "error").unwrap(), i % 4);
        }
        // The generator has no entropy remaining at this point.
        assert_eq!(gen.len(), 0);
        assert_eq!(
            gen.int_in_range(0..=3, || "error"),
            Err(MutationError::User("Ran out of entropy".to_string()))
        );
    }

    /// Test the error when there are no possible choices.
    #[test]
    pub fn test_generator_no_choices() {
        let mut gen = crate::generator::Generator::from(&[0, 1, 2, 3, 4, 5][..]);
        let choices: [usize; 0] = [];
        assert_eq!(
            gen.choose(&choices, || "error"),
            Err(MutationError::User("Empty choices".to_string()))
        );
    }

    /// Test that a generator can be used to choose one value.
    #[test]
    pub fn test_generator_one_choices() {
        let mut gen = crate::generator::Generator::from(&[0, 1, 2, 3, 4, 5][..]);
        let choices = [0];
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &0);
    }

    /// Test that a generator can be used to choose between two values.
    #[test]
    pub fn test_generator_choose_two_values() {
        let mut gen = crate::generator::Generator::from(&[0, 1, 2, 3, 4, 5][..]);
        let choices = [0, 1];
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &0);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &1);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &0);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &1);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &0);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &1);

        // This is the only case where we can't choose a value, because we have 2 choices and 6 bytes.
        assert_eq!(
            gen.choose(&choices, || "error"),
            Err(MutationError::User("Ran out of entropy".to_string()))
        );
    }

    /// Test that a generator can be used to choose between three values.
    #[test]
    pub fn test_generator_choose_three_values() {
        let mut gen = crate::generator::Generator::from(&[0, 1, 2, 3, 4, 5][..]);
        let choices = [0, 1, 2];
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &0);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &1);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &2);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &0);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &1);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &2);

        // This is the only case where we can't choose a value, because we have 3 choices and 6 bytes.
        assert_eq!(
            gen.choose(&choices, || "error"),
            Err(MutationError::User("Ran out of entropy".to_string()))
        );
    }

    /// Test that a generator can be used to choose between three values with
    /// randomization.
    #[test]
    pub fn test_generator_choose_three_values_random() {
        let mut u_data = [0, 1, 2, 3, 4, 5];

        // Seeded random so we get deterministic results.
        let mut rng = rand::rngs::StdRng::seed_from_u64(0);
        u_data.shuffle(&mut rng);

        let mut gen = crate::generator::Generator::from(&u_data[..]);

        let choices = [0, 1, 2];
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &1);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &2);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &2);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &1);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &0);
        assert_eq!(gen.choose(&choices, || "error").unwrap(), &0);

        // This is the only case where we can't choose a value, because we have 3 choices and 6 bytes.
        assert_eq!(
            gen.choose(&choices, || "error"),
            Err(MutationError::User("Ran out of entropy".to_string()))
        );
    }

    /// Test that a generator can choose a single item even if there is no entropy.
    #[test]
    pub fn test_generator_choose_single_without_entropy() {
        let mut gen = crate::generator::Generator::from(&[0][..]);
        let choices = [0];
        for _ in 0..10 {
            assert_eq!(gen.choose(&choices, || "error").unwrap(), &0);
        }

        // The generator has not had any entropy consumed by this point.
        assert_eq!(gen.len(), 1);
    }
}
