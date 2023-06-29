//! Some predicates borrowed from predicates-rs
//! https://github.com/assert-rs/predicates-rs

use std::{
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

use crate::{fact::check_raw, *};

/// A constraint which is always met
pub fn always() -> BoolFact {
    BoolFact(true, "always".to_string())
}

/// A constraint which is never met
pub fn never<S: ToString>(context: S) -> BoolFact {
    BoolFact(false, context.to_string())
}

/// Specifies an equality constraint
pub fn eq<S, T>(context: S, constant: T) -> EqFact<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq,
{
    EqFact {
        context: context.to_string(),
        constant,
        op: EqOp::Equal,
        _phantom: PhantomData,
    }
}

/// Specifies an equality constraint with no context
pub fn eq_<T>(constant: T) -> EqFact<T>
where
    T: std::fmt::Debug + PartialEq,
{
    eq("eq", constant)
}

/// Specifies an inequality constraint
pub fn ne<S, T>(context: S, constant: T) -> EqFact<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq,
{
    EqFact {
        context: context.to_string(),
        constant,
        op: EqOp::NotEqual,
        _phantom: PhantomData,
    }
}

/// Specifies an inequality constraint with no context
pub fn ne_<T>(constant: T) -> EqFact<T>
where
    T: std::fmt::Debug + PartialEq,
{
    ne("ne", constant)
}

/// Specifies an equality constraint between two items in a tuple
pub fn same<T>() -> SameFact<T>
where
    T: std::fmt::Debug + PartialEq,
{
    SameFact {
        op: EqOp::Equal,
        _phantom: PhantomData,
    }
}

/// Specifies an inequality constraint between two items in a tuple
pub fn different<T>() -> SameFact<T>
where
    T: std::fmt::Debug + PartialEq,
{
    SameFact {
        op: EqOp::NotEqual,
        _phantom: PhantomData,
    }
}

/// Specifies a membership constraint
pub fn in_slice<'a, S, T>(context: S, slice: &'a [T]) -> InSliceFact<'a, T>
where
    S: ToString,
    T: 'a + PartialEq + std::fmt::Debug + Clone,
{
    InSliceFact {
        context: context.to_string(),
        slice,
    }
}

/// Specifies a membership constraint
pub fn in_slice_<'a, T>(slice: &'a [T]) -> InSliceFact<'a, T>
where
    T: 'a + PartialEq + std::fmt::Debug + Clone,
{
    in_slice("in_slice", slice)
}

/// Specifies a range constraint
pub fn in_range<S, R, T>(context: S, range: R) -> InRangeFact<R, T>
where
    S: ToString,
    R: RangeBounds<T> + std::fmt::Debug,
    T: PartialEq
        + PartialOrd
        + Ord
        + Clone
        + std::fmt::Debug
        + num::traits::Euclid
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + num::Bounded
        + num::One,
{
    InRangeFact {
        context: context.to_string(),
        range,
        phantom: PhantomData,
    }
}

/// Specifies a range constraint
pub fn in_range_<R, T>(range: R) -> InRangeFact<R, T>
where
    R: RangeBounds<T> + std::fmt::Debug,
    T: PartialEq
        + PartialOrd
        + Ord
        + Clone
        + std::fmt::Debug
        + num::traits::Euclid
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + num::Bounded
        + num::One,
{
    in_range("in_range", range)
}

/// Specifies that a value should be increasing by 1 at every check/mutation
pub fn consecutive_int<S, T>(context: S, initial: T) -> ConsecutiveIntFact<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq + num::PrimInt,
{
    ConsecutiveIntFact {
        context: context.to_string(),
        counter: initial,
    }
}

/// Specifies that a value should be increasing by 1 at every check/mutation,
/// with no context given
pub fn consecutive_int_<T>(initial: T) -> ConsecutiveIntFact<T>
where
    T: std::fmt::Debug + PartialEq + num::PrimInt,
{
    consecutive_int("consecutive_int", initial)
}

/// Combines two constraints so that either one may be satisfied
pub fn or<'a, A, T, S, Item>(context: S, a: A, b: T) -> OrFact<'a, A, T, Item>
where
    S: ToString,
    A: Fact<'a, Item>,
    T: Fact<'a, Item>,
    Item: Bounds<'a>,
{
    OrFact {
        context: context.to_string(),
        a,
        b,
        _phantom: PhantomData,
    }
}

/// Negates a fact
// TODO: `not` in particular would really benefit from Facts having accessible
// labels, since currently you can only get context about why a `not` fact passed,
// not why it fails.
pub fn not<'a, F, S, T>(context: S, fact: F) -> NotFact<'a, F, T>
where
    S: ToString,
    F: Fact<'a, T>,
    T: Bounds<'a>,
{
    NotFact {
        context: context.to_string(),
        fact,
        _phantom: PhantomData,
    }
}

/// Negates a fact, with no context given
pub fn not_<'a, F, T>(fact: F) -> NotFact<'a, F, T>
where
    F: Fact<'a, T>,
    T: Bounds<'a>,
{
    not("not", fact)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoolFact(bool, String);

impl<'a, T> Fact<'a, T> for BoolFact
where
    T: Bounds<'a> + PartialEq + Clone,
{
    #[tracing::instrument(fields(fact = "bool"), skip(self, g))]
    fn mutate(&self, t: T, g: &mut Generator<'_>) -> Mutation<T> {
        if !self.0 {
            g.fail("never() encountered.")?;
        }
        Ok(t)
    }

    #[tracing::instrument(fields(fact = "bool"), skip(self))]
    fn advance(&mut self, _: &T) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EqFact<T> {
    context: String,
    op: EqOp,
    constant: T,
    _phantom: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EqOp {
    Equal,
    NotEqual,
}

impl<'a, T> Fact<'a, T> for EqFact<T>
where
    T: Bounds<'a> + PartialEq + Clone,
{
    #[tracing::instrument(fields(fact = "eq"), skip(self, g))]
    fn mutate(&self, mut obj: T, g: &mut Generator<'a>) -> Mutation<T> {
        let constant = self.constant.clone();
        match self.op {
            EqOp::Equal => {
                if obj != constant {
                    g.fail(format!(
                        "{}: expected {:?} == {:?}",
                        self.context, obj, constant
                    ))?;
                    obj = constant;
                }
            }
            EqOp::NotEqual => loop {
                if obj != constant {
                    break;
                }
                obj = g.arbitrary(format!(
                    "{}: expected {:?} != {:?}",
                    self.context, obj, constant
                ))?;
            },
        }
        Ok(obj)
    }

    #[tracing::instrument(fields(fact = "eq"), skip(self))]
    fn advance(&mut self, _: &T) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SameFact<T> {
    op: EqOp,
    _phantom: PhantomData<T>,
}

impl<'a, T> Fact<'a, (T, T)> for SameFact<T>
where
    T: Bounds<'a> + PartialEq + Clone,
{
    #[tracing::instrument(fields(fact = "same"), skip(self, g))]
    fn mutate(&self, mut obj: (T, T), g: &mut Generator<'a>) -> Mutation<(T, T)> {
        match self.op {
            EqOp::Equal => {
                if obj.0 != obj.1 {
                    g.fail(format!("must be same: expected {:?} == {:?}", obj.0, obj.1))?;
                    obj.0 = obj.1.clone();
                }
            }
            EqOp::NotEqual => loop {
                if obj.0 != obj.1 {
                    break;
                }
                obj.0 = g.arbitrary(format!(
                    "must be different: expected {:?} != {:?}",
                    obj.0, obj.1
                ))?;
            },
        }
        Ok(obj)
    }

    #[tracing::instrument(fields(fact = "same"), skip(self))]
    fn advance(&mut self, _: &(T, T)) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InSliceFact<'a, T>
where
    T: 'a + PartialEq + std::fmt::Debug + Clone,
    // I: Iterator<Item = &'a T>,
{
    context: String,
    slice: &'a [T],
}

impl<'a, 'b: 'a, T> Fact<'a, T> for InSliceFact<'b, T>
where
    T: 'b + Bounds<'a> + Clone,
    // I: Iterator<Item = &'b T>,
{
    fn mutate(&self, obj: T, g: &mut Generator<'a>) -> Mutation<T> {
        Ok(if !self.slice.contains(&obj) {
            g.choose(
                self.slice,
                format!(
                    "{}: expected {:?} to be contained in {:?}",
                    self.context, obj, self.slice
                ),
            )?
            .to_owned()
        } else {
            obj
        })
    }

    fn advance(&mut self, _: &T) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InRangeFact<R, T>
where
    R: RangeBounds<T> + std::fmt::Debug,
    T: PartialEq
        + PartialOrd
        + Ord
        + Clone
        + std::fmt::Debug
        + num::traits::Euclid
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + num::Bounded
        + num::One,
{
    context: String,
    range: R,
    phantom: PhantomData<T>,
}

impl<'a, R, T> Fact<'a, T> for InRangeFact<R, T>
where
    R: Send + Sync + RangeBounds<T> + std::fmt::Debug,
    T: Bounds<'a>
        + PartialEq
        + PartialOrd
        + Ord
        + Clone
        + std::fmt::Debug
        + num::traits::Euclid
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + num::Bounded
        + num::One,
{
    fn mutate(&self, mut obj: T, g: &mut Generator<'a>) -> Mutation<T> {
        if !self.range.contains(&obj) {
            let rand = g.arbitrary(format!(
                "{}: expected {:?} to be contained in {:?}",
                self.context, obj, self.range
            ))?;
            obj = match (self.range.start_bound(), self.range.end_bound()) {
                (Bound::Unbounded, Bound::Unbounded) => rand,
                (Bound::Included(a), Bound::Included(b)) if b.clone() - a.clone() >= T::one() => {
                    a.clone() + rand.rem_euclid(&(b.clone() - a.clone()))
                }
                (Bound::Included(a), Bound::Excluded(b)) if b.clone() - a.clone() > T::one() => {
                    a.clone() + rand.rem_euclid(&(b.clone() - a.clone()))
                }
                (Bound::Excluded(a), Bound::Included(b)) if b.clone() - a.clone() > T::one() => {
                    b.clone() - rand.rem_euclid(&(b.clone() - a.clone()))
                }
                (Bound::Unbounded, Bound::Excluded(b)) => {
                    T::min_value() + rand.rem_euclid(&(b.clone() - T::min_value()))
                }
                (Bound::Included(a), Bound::Unbounded) => {
                    a.clone() + rand.rem_euclid(&(T::max_value() - a.clone()))
                }
                _ => panic!("Range not yet supported, sorry! {:?}", self.range),
            };
        }
        Ok(obj)
    }

    fn advance(&mut self, _: &T) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsecutiveIntFact<T> {
    context: String,
    counter: T,
}

impl<'a, T> Fact<'a, T> for ConsecutiveIntFact<T>
where
    T: Bounds<'a> + num::PrimInt,
{
    #[tracing::instrument(fields(fact = "consecutive_int"), skip(self, g))]
    fn mutate(&self, mut obj: T, g: &mut Generator<'a>) -> Mutation<T> {
        if obj != self.counter {
            g.fail(&self.context)?;
            obj = self.counter.clone();
        }
        Ok(obj)
    }

    #[tracing::instrument(fields(fact = "consecutive_int"), skip(self))]
    fn advance(&mut self, _: &T) {
        self.counter = self.counter.checked_add(&T::from(1).unwrap()).unwrap();
    }
}

/// Fact that combines two `Fact`s, returning the OR of the results.
///
/// This is created by the `or` function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrFact<'a, M1, M2, Item>
where
    M1: Fact<'a, Item>,
    M2: Fact<'a, Item>,
    Item: ?Sized + Bounds<'a>,
{
    context: String,
    pub(crate) a: M1,
    pub(crate) b: M2,
    _phantom: PhantomData<&'a Item>,
}

impl<'a, P1, P2, T> Fact<'a, T> for OrFact<'a, P1, P2, T>
where
    P1: Fact<'a, T> + Fact<'a, T>,
    P2: Fact<'a, T> + Fact<'a, T>,
    T: Bounds<'a>,
{
    fn mutate(&self, obj: T, g: &mut Generator<'a>) -> Mutation<T> {
        use rand::{thread_rng, Rng};

        let a = check_raw(&self.a, &obj).is_ok();
        let b = check_raw(&self.b, &obj).is_ok();
        match (a, b) {
            (true, _) => Ok(obj),
            (_, true) => Ok(obj),
            (false, false) => {
                g.fail(format!(
                    "expected either one of the following conditions to be met:
    condition 1: {:#?}
    condition 2: {:#?}",
                    a, b
                ))?;
                if thread_rng().gen::<bool>() {
                    self.a.mutate(obj, g)
                } else {
                    self.b.mutate(obj, g)
                }
            }
        }
    }

    fn advance(&mut self, _: &T) {}
}

#[derive(Debug, Clone)]
pub struct NotFact<'a, F, T>
where
    F: Fact<'a, T>,
    T: Bounds<'a>,
{
    context: String,
    fact: F,
    _phantom: PhantomData<&'a T>,
}

impl<'a, F, T> Fact<'a, T> for NotFact<'a, F, T>
where
    F: Fact<'a, T>,
    T: Bounds<'a>,
{
    fn mutate(&self, mut obj: T, g: &mut Generator<'a>) -> Mutation<T> {
        for _ in 0..BRUTE_ITERATION_LIMIT {
            if check_raw(&self.fact, &obj).is_err() {
                break;
            }
            obj = g
                .arbitrary(format!("not({})", self.context.clone()))
                .unwrap();
        }
        Ok(obj)
    }

    fn advance(&mut self, _: &T) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;

    #[test]
    fn test_eq() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let eq1 = seq_(eq("must be 1", 1));

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
        let mut either = or("can be 1 or 2", eq1, eq2);

        let ones = seq_(either.clone()).build(&mut g);
        seq_(either.clone()).check(&ones).unwrap();
        assert!(ones.iter().all(|x| *x == 1 || *x == 2));

        assert_eq!(either.check(&3).result().unwrap().unwrap_err().len(), 1);
    }

    #[test]
    fn test_not() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        let eq1 = eq("must be 1", 1);
        let not1 = seq_(not_(eq1));

        let nums = not1.clone().build(&mut g);
        not1.clone().check(&nums).unwrap();

        assert!(nums.iter().all(|x| *x != 1));
    }

    #[test]
    fn test_same() {
        observability::test_run().ok();
        let mut g = utils::random_generator();

        {
            let f = seq_(same::<u8>());
            let nums = f.clone().build(&mut g);
            f.clone().check(&nums).unwrap();
            assert!(nums.iter().all(|(a, b)| a == b));
        }
        {
            let f = seq_(different::<u8>());
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

        let nonpositive1 = seq_(not_(positive1));
        let nonpositive2 = seq_(not_(positive2));

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
