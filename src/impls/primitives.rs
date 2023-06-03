//! Some predicates borrowed from predicates-rs
//! https://github.com/assert-rs/predicates-rs

use std::{
    borrow::Borrow,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

use crate::{fact::*, Check, BRUTE_ITERATION_LIMIT};

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
pub fn same<S, T>(context: S) -> SameFact<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq,
{
    SameFact {
        context: context.to_string(),
        op: EqOp::Equal,
        _phantom: PhantomData,
    }
}

/// Specifies an equality constraint between two items in a tuple
pub fn different<S, T>(context: S) -> SameFact<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq,
{
    SameFact {
        context: context.to_string(),
        op: EqOp::NotEqual,
        _phantom: PhantomData,
    }
}

/// Specifies a membership constraint
pub fn in_iter<'a, I, S, T>(context: S, iter: I) -> InIterFact<'a, T>
where
    S: ToString,
    T: 'a + PartialEq + std::fmt::Debug + Clone,
    I: IntoIterator<Item = &'a T>,
{
    use std::iter::FromIterator;
    InIterFact {
        context: context.to_string(),
        inner: Vec::from_iter(iter),
    }
}

/// Specifies a membership constraint
pub fn in_iter_<'a, I, T>(iter: I) -> InIterFact<'a, T>
where
    T: 'a + PartialEq + std::fmt::Debug + Clone,
    I: IntoIterator<Item = &'a T>,
{
    in_iter("in_iter", iter)
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
    fn check(&self, _: &T) -> Check {
        if self.0 {
            Vec::with_capacity(0)
        } else {
            vec![format!("never() encountered: {}", self.1)]
        }
        .into()
    }

    fn mutate(&self, t: T, _: &mut arbitrary::Unstructured<'_>) -> T {
        if !self.0 {
            panic!("never() cannot be used for mutation.")
        }
        t
    }

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
    fn check(&self, obj: &T) -> Check {
        let constant = self.constant.borrow();
        match self.op {
            EqOp::Equal if obj != constant => vec![format!(
                "{}: expected {:?} == {:?}",
                self.context, obj, constant
            )],
            EqOp::NotEqual if obj == constant => vec![format!(
                "{}: expected {:?} != {:?}",
                self.context, obj, constant
            )],
            _ => Vec::with_capacity(0),
        }
        .into()
    }

    #[allow(unused_assignments)]
    fn mutate(&self, mut obj: T, u: &mut arbitrary::Unstructured<'a>) -> T {
        let constant = self.constant.clone();
        match self.op {
            EqOp::Equal => obj = constant,
            EqOp::NotEqual => loop {
                obj = T::arbitrary(u).unwrap();
                if obj != constant {
                    break;
                }
            },
        }
        self.check(&obj)
            .result()
            .expect("there's a bug in EqFact::mutate");
        obj
    }

    fn advance(&mut self, _: &T) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SameFact<T> {
    context: String,
    op: EqOp,
    _phantom: PhantomData<T>,
}

impl<'a, T> Fact<'a, (T, T)> for SameFact<T>
where
    T: Bounds<'a> + PartialEq + Clone,
{
    fn check(&self, obj: &(T, T)) -> Check {
        let (a, b) = obj;
        match self.op {
            EqOp::Equal if a != b => vec![format!("{}: expected {:?} == {:?}", self.context, a, b)],
            EqOp::NotEqual if a == b => {
                vec![format!("{}: expected {:?} != {:?}", self.context, a, b)]
            }
            _ => Vec::with_capacity(0),
        }
        .into()
    }

    #[allow(unused_assignments)]
    fn mutate(&self, mut obj: (T, T), u: &mut arbitrary::Unstructured<'a>) -> (T, T) {
        match self.op {
            EqOp::Equal => obj.0 = obj.1.clone(),
            EqOp::NotEqual => loop {
                obj.0 = T::arbitrary(u).unwrap();
                if obj.0 != obj.1 {
                    break;
                }
            },
        }
        self.check(&obj)
            .result()
            .expect("there's a bug in EqFact::mutate");
        obj
    }

    fn advance(&mut self, _: &(T, T)) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InIterFact<'a, T>
where
    T: PartialEq + std::fmt::Debug + Clone,
{
    context: String,
    inner: Vec<&'a T>,
}

impl<'a, T> Fact<'a, T> for InIterFact<'_, T>
where
    T: Bounds<'a> + Clone,
{
    fn check(&self, obj: &T) -> Check {
        if self.inner.contains(&obj) {
            Vec::with_capacity(0)
        } else {
            vec![format!(
                "{}: expected {:?} to be contained in {:?}",
                self.context, obj, self.inner
            )]
        }
        .into()
    }

    #[allow(unused_assignments)]
    fn mutate(&self, mut obj: T, u: &mut arbitrary::Unstructured<'a>) -> T {
        obj = (*u.choose(self.inner.as_slice()).unwrap()).to_owned();
        self.check(&obj)
            .result()
            .expect("there's a bug in InIterFact::mutate");
        obj
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
    R: RangeBounds<T> + std::fmt::Debug,
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
    fn check(&self, obj: &T) -> Check {
        if self.range.contains(&obj) {
            Vec::with_capacity(0)
        } else {
            vec![format!(
                "{}: expected {:?} to be contained in {:?}",
                self.context, obj, self.range
            )]
        }
        .into()
    }

    #[allow(unused_assignments)]
    fn mutate(&self, mut obj: T, u: &mut arbitrary::Unstructured<'a>) -> T {
        let rand = T::arbitrary(u).unwrap();
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
        self.check(&obj)
            .result()
            .expect("there's a bug in InRangeFact::mutate");
        obj
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
    fn check(&self, obj: &T) -> Check {
        Check::check(*obj == self.counter, self.context.clone())
    }

    #[allow(unused_assignments)]
    fn mutate(&self, mut obj: T, _: &mut arbitrary::Unstructured<'a>) -> T {
        obj = self.counter.clone();
        obj
    }

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
    fn check(&self, obj: &T) -> Check {
        let a = self.a.check(obj).result();
        let b = self.b.check(obj).result();
        match (a, b) {
            (Err(a), Err(b)) => vec![format!(
                "expected either one of the following conditions to be met:
condition 1: {:#?}
condition 2: {:#?}",
                a, b
            )]
            .into(),
            _ => Check::pass(),
        }
    }

    fn mutate(&self, obj: T, u: &mut arbitrary::Unstructured<'a>) -> T {
        if *u.choose(&[true, false]).unwrap() {
            self.a.mutate(obj, u)
        } else {
            self.b.mutate(obj, u)
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
    fn check(&self, obj: &T) -> Check {
        Check::check(
            self.fact.check(obj).is_err(),
            format!("not({})", self.context.clone()),
        )
    }

    fn mutate(&self, mut obj: T, u: &mut arbitrary::Unstructured<'a>) -> T {
        for _ in 0..BRUTE_ITERATION_LIMIT {
            if self.fact.check(&obj).is_err() {
                break;
            }
            obj = T::arbitrary(u).unwrap();
        }
        obj
    }

    fn advance(&mut self, _: &T) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_seq, check_seq, utils};

    #[test]
    fn test_eq() {
        observability::test_run().ok();
        let mut u = utils::unstructured_noise();

        let eq1 = eq("must be 1", 1);

        let ones = build_seq(&mut u, 3, eq1.clone());
        check_seq(ones.as_slice(), eq1.clone()).unwrap();

        assert!(ones.iter().all(|x| *x == 1));
    }

    #[test]
    fn test_or() {
        observability::test_run().ok();
        let mut u = utils::unstructured_noise();

        let eq1 = eq("must be 1", 1);
        let eq2 = eq("must be 2", 2);
        let either = or("can be 1 or 2", eq1, eq2);

        let ones = build_seq(&mut u, 10, either.clone());
        check_seq(ones.as_slice(), either.clone()).unwrap();
        assert!(ones.iter().all(|x| *x == 1 || *x == 2));

        assert_eq!(either.check(&3).result().unwrap_err().len(), 1);
    }

    #[test]
    fn test_not() {
        observability::test_run().ok();
        let mut u = utils::unstructured_noise();

        let eq1 = eq("must be 1", 1);
        let not1 = not_(eq1);

        let nums = build_seq(&mut u, 10, not1.clone());
        check_seq(nums.as_slice(), not1.clone()).unwrap();

        assert!(nums.iter().all(|x| *x != 1));
    }

    #[test]
    fn test_same() {
        observability::test_run().ok();
        let mut u = utils::unstructured_noise();

        {
            let f = same::<_, u8>("must be same");
            let nums = build_seq(&mut u, 10, f.clone());
            check_seq(nums.as_slice(), f.clone()).unwrap();
            assert!(nums.iter().all(|(a, b)| a == b));
        }
        {
            let f = different::<_, u8>("must be different");
            let nums = build_seq(&mut u, 10, f.clone());
            check_seq(nums.as_slice(), f.clone()).unwrap();
            assert!(nums.iter().all(|(a, b)| a != b));
        }
    }

    #[test]
    fn test_in_range() {
        observability::test_run().ok();
        let mut u = utils::unstructured_noise();

        let positive1 = in_range("must be positive", 1..=i32::MAX);
        let positive2 = in_range("must be positive", 1..);
        let smallish = in_range("must be small in magnitude", -10..100);
        let over9000 = in_range("must be over 9000", 9001..);
        let under9000 = in_range("must be under 9000 (and no less than zero)", ..9000u32);

        let nonpositive1 = not_(positive1);
        let nonpositive2 = not_(positive2);

        let smallish_nums = build_seq(&mut u, 100, smallish.clone());
        let over9000_nums = build_seq(&mut u, 100, over9000.clone());
        let under9000_nums = build_seq(&mut u, 100, under9000.clone());
        let nonpositive1_nums = build_seq(&mut u, 20, nonpositive1.clone());
        let nonpositive2_nums = build_seq(&mut u, 20, nonpositive2.clone());

        dbg!(&under9000_nums);

        check_seq(smallish_nums.as_slice(), smallish.clone()).unwrap();
        check_seq(over9000_nums.as_slice(), over9000.clone()).unwrap();
        check_seq(under9000_nums.as_slice(), under9000.clone()).unwrap();
        check_seq(nonpositive1_nums.as_slice(), nonpositive1.clone()).unwrap();
        check_seq(nonpositive2_nums.as_slice(), nonpositive2.clone()).unwrap();
        assert!(nonpositive1_nums.iter().all(|x| *x <= 0));
    }
}
