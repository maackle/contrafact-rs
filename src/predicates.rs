//! Some predicates borrowed from predicates-rs
//! https://github.com/assert-rs/predicates-rs

use std::marker::PhantomData;

use crate::fact::*;

/// A constraint which is always met
pub fn always() -> BoolFact {
    BoolFact(true)
}

/// A constraint which is never met
pub fn never() -> BoolFact {
    BoolFact(false)
}

/// Specifies an equality constraint
pub fn eq<S, T>(reason: S, constant: &T) -> EqFact<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq,
{
    EqFact {
        reason: reason.to_string(),
        constant,
        op: EqOp::Equal,
    }
}

/// Specifies an inequality constraint
pub fn ne<S, T>(reason: S, constant: &T) -> EqFact<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq,
{
    EqFact {
        reason: reason.to_string(),
        constant,
        op: EqOp::NotEqual,
    }
}

/// Specifies a membership constraint
pub fn in_iter<'a, I, S, T>(reason: S, iter: I) -> InFact<'a, T>
where
    S: ToString,
    T: 'a + PartialEq + std::fmt::Debug,
    I: IntoIterator<Item = &'a T>,
{
    use std::iter::FromIterator;
    InFact {
        reason: reason.to_string(),
        inner: Vec::from_iter(iter),
    }
}

/// Specifies an equality constraint
pub fn consecutive_int<S, T>(reason: S, initial: T) -> ConsecutiveIntFact<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq + num::PrimInt,
{
    ConsecutiveIntFact {
        reason: reason.to_string(),
        counter: initial,
    }
}

/// Combines two constraints so that either one may be satisfied
pub fn or<A, B, S, Item>(reason: S, a: A, b: B) -> OrFact<A, B, Item>
where
    S: ToString,
    A: Fact<Item>,
    B: Fact<Item>,
    Item: Bounds,
{
    OrFact {
        reason: reason.to_string(),
        a,
        b,
        _phantom: PhantomData,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoolFact(bool);

impl<T> Fact<T> for BoolFact
where
    T: Bounds + PartialEq,
{
    fn check(&mut self, _: &T) -> CheckResult {
        if self.0 {
            Vec::with_capacity(0)
        } else {
            vec![format!("never() always fails")]
        }
        .into()
    }

    fn mutate(&mut self, _: &mut T, _: &mut arbitrary::Unstructured<'static>) {
        if !self.0 {
            panic!("never() cannot be used for mutation.")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EqFact<'a, T> {
    reason: String,
    op: EqOp,
    constant: &'a T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EqOp {
    Equal,
    NotEqual,
}

impl<T> Fact<T> for EqFact<'_, T>
where
    T: Bounds + PartialEq,
{
    fn check(&mut self, obj: &T) -> CheckResult {
        match self.op {
            EqOp::Equal if obj != self.constant => vec![format!(
                "{}: expected {:?} == {:?}",
                self.reason, obj, self.constant
            )],
            EqOp::NotEqual if obj == self.constant => vec![format!(
                "{}: expected {:?} != {:?}",
                self.reason, obj, self.constant
            )],
            _ => Vec::with_capacity(0),
        }
        .into()
    }

    fn mutate(&mut self, obj: &mut T, u: &mut arbitrary::Unstructured<'static>) {
        match self.op {
            EqOp::Equal => *obj = self.constant.clone(),
            EqOp::NotEqual => loop {
                *obj = T::arbitrary(u).unwrap();
                if obj != self.constant {
                    break;
                }
            },
        }
        self.check(obj)
            .ok()
            .expect("there's a bug in EqFact::mutate");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InFact<'a, T>
where
    T: PartialEq + std::fmt::Debug,
{
    reason: String,
    inner: Vec<&'a T>,
}

impl<T> Fact<T> for InFact<'_, T>
where
    T: Bounds,
{
    fn check(&mut self, obj: &T) -> CheckResult {
        if self.inner.contains(&obj) {
            Vec::with_capacity(0)
        } else {
            vec![format!(
                "{}: expected {:?} to be contained in {:?}",
                self.reason, obj, self.inner
            )]
        }
        .into()
    }

    fn mutate(&mut self, obj: &mut T, u: &mut arbitrary::Unstructured<'static>) {
        *obj = (*u.choose(self.inner.as_slice()).unwrap()).to_owned();
        self.check(obj)
            .ok()
            .expect("there's a bug in InFact::mutate");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsecutiveIntFact<T> {
    reason: String,
    counter: T,
}

impl<T> Fact<T> for ConsecutiveIntFact<T>
where
    T: Bounds + num::PrimInt,
{
    fn check(&mut self, obj: &T) -> CheckResult {
        let result = if *obj == self.counter {
            CheckResult::pass()
        } else {
            vec![self.reason.clone()].into()
        };
        self.counter = self.counter.checked_add(&T::from(1).unwrap()).unwrap();
        result
    }

    fn mutate(&mut self, obj: &mut T, _: &mut arbitrary::Unstructured<'static>) {
        *obj = self.counter.clone();
        self.counter = self.counter.checked_add(&T::from(1).unwrap()).unwrap();
    }
}

/// Fact that combines two `Fact`s, returning the OR of the results.
///
/// This is created by the `or` function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrFact<M1, M2, Item>
where
    M1: Fact<Item>,
    M2: Fact<Item>,
    Item: ?Sized + Bounds,
{
    reason: String,
    pub(crate) a: M1,
    pub(crate) b: M2,
    _phantom: PhantomData<Item>,
}

impl<P1, P2, T> Fact<T> for OrFact<P1, P2, T>
where
    P1: Fact<T> + Fact<T>,
    P2: Fact<T> + Fact<T>,
    T: Bounds,
{
    fn check(&mut self, obj: &T) -> CheckResult {
        let a = self.a.check(obj).ok();
        let b = self.b.check(obj).ok();
        match (a, b) {
            (Err(a), Err(b)) => vec![format!(
                "expected either one of the following conditions to be met:
condition 1: {:#?}
condition 2: {:#?}",
                a, b
            )]
            .into(),
            _ => CheckResult::pass(),
        }
    }

    fn mutate(&mut self, obj: &mut T, u: &mut arbitrary::Unstructured<'static>) {
        if *u.choose(&[true, false]).unwrap() {
            self.a.mutate(obj, u);
        } else {
            self.b.mutate(obj, u);
        }
        self.check(obj)
            .ok()
            .expect("there's a bug in OrFact::mutate");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_seq, check_seq, NOISE};
    use arbitrary::Unstructured;

    #[test]
    fn test_eq() {
        observability::test_run().ok();
        let mut u = Unstructured::new(&NOISE);

        let eq1 = eq("must be 1", &1);

        let ones = build_seq(&mut u, 3, eq1.clone());
        check_seq(ones.as_slice(), eq1.clone()).unwrap();

        assert!(ones.iter().all(|x| *x == 1));
    }

    #[test]
    fn test_or() {
        observability::test_run().ok();
        let mut u = Unstructured::new(&NOISE);

        let eq1 = eq("must be 1", &1);
        let eq2 = eq("must be 2", &2);
        let mut either = or("can be 1 or 2", eq1, eq2);

        let ones = build_seq(&mut u, 10, either.clone());
        check_seq(ones.as_slice(), either.clone()).unwrap();
        assert!(ones.iter().all(|x| *x == 1 || *x == 2));

        assert_eq!(either.check(&3).ok().unwrap_err().len(), 1);
    }
}
