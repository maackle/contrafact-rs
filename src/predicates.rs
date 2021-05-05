//! Some predicates borrowed from predicates-rs
//! https://github.com/assert-rs/predicates-rs

use std::marker::PhantomData;

use crate::constraint::*;

/// A constraint which is always met
pub fn always() -> BoolConstraint {
    BoolConstraint(true)
}

/// A constraint which is never met
pub fn never() -> BoolConstraint {
    BoolConstraint(false)
}

/// Specifies an equality constraint
pub fn eq<S, T>(reason: S, constant: &T) -> EqConstraint<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq,
{
    EqConstraint {
        reason: reason.to_string(),
        constant,
        op: EqOp::Equal,
    }
}

/// Specifies an inequality constraint
pub fn ne<S, T>(reason: S, constant: &T) -> EqConstraint<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq,
{
    EqConstraint {
        reason: reason.to_string(),
        constant,
        op: EqOp::NotEqual,
    }
}

/// Specifies a membership constraint
pub fn in_iter<'a, I, S, T>(reason: S, iter: I) -> InConstraint<'a, T>
where
    S: ToString,
    T: 'a + PartialEq + std::fmt::Debug,
    I: IntoIterator<Item = &'a T>,
{
    use std::iter::FromIterator;
    InConstraint {
        reason: reason.to_string(),
        inner: Vec::from_iter(iter),
    }
}

/// Combines two constraints so that either one may be satisfied
pub fn or<A, B, S, Item>(reason: S, a: A, b: B) -> OrConstraint<A, B, Item>
where
    S: ToString,
    A: Constraint<Item>,
    B: Constraint<Item>,
    Item: Bounds,
{
    OrConstraint {
        reason: reason.to_string(),
        a,
        b,
        _phantom: PhantomData,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoolConstraint(bool);

impl<T> Constraint<T> for BoolConstraint
where
    T: Bounds + PartialEq,
{
    fn check(&self, _: &T) -> CheckResult {
        if self.0 {
            Vec::with_capacity(0)
        } else {
            vec![format!("never() always fails")]
        }
        .into()
    }

    fn mutate(&self, _: &mut T, _: &mut arbitrary::Unstructured<'static>) {
        if !self.0 {
            panic!("never() cannot be used for mutation.")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EqConstraint<'a, T> {
    reason: String,
    op: EqOp,
    constant: &'a T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EqOp {
    Equal,
    NotEqual,
}

impl<T> Constraint<T> for EqConstraint<'_, T>
where
    T: Bounds + PartialEq,
{
    fn check(&self, obj: &T) -> CheckResult {
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

    fn mutate(&self, obj: &mut T, u: &mut arbitrary::Unstructured<'static>) {
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
            .expect("there's a bug in EqConstraint::mutate");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InConstraint<'a, T>
where
    T: PartialEq + std::fmt::Debug,
{
    reason: String,
    inner: Vec<&'a T>,
}

impl<T> Constraint<T> for InConstraint<'_, T>
where
    T: Bounds,
{
    fn check(&self, obj: &T) -> CheckResult {
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

    fn mutate(&self, obj: &mut T, u: &mut arbitrary::Unstructured<'static>) {
        *obj = (*u.choose(self.inner.as_slice()).unwrap()).to_owned();
        self.check(obj)
            .ok()
            .expect("there's a bug in InConstraint::mutate");
    }
}

/// Constraint that combines two `Constraint`s, returning the OR of the results.
///
/// This is created by the `or` function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrConstraint<M1, M2, Item>
where
    M1: Constraint<Item>,
    M2: Constraint<Item>,
    Item: ?Sized + Bounds,
{
    reason: String,
    pub(crate) a: M1,
    pub(crate) b: M2,
    _phantom: PhantomData<Item>,
}

impl<P1, P2, T> Constraint<T> for OrConstraint<P1, P2, T>
where
    P1: Constraint<T> + Constraint<T>,
    P2: Constraint<T> + Constraint<T>,
    T: Bounds,
{
    fn check(&self, obj: &T) -> CheckResult {
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

    fn mutate(&self, obj: &mut T, u: &mut arbitrary::Unstructured<'static>) {
        if *u.choose(&[true, false]).unwrap() {
            self.a.mutate(obj, u);
        } else {
            self.b.mutate(obj, u);
        }
        self.check(obj)
            .ok()
            .expect("there's a bug in OrConstraint::mutate");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_seq, check_seq, Fact, NOISE};
    use arbitrary::Unstructured;

    #[test]
    fn test_eq() {
        observability::test_run().ok();
        let mut u = Unstructured::new(&NOISE);

        let eq1 = eq("must be 1", &1).to_fact();

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
        let either = or("can be 1 or 2", eq1, eq2).to_fact();

        let ones = build_seq(&mut u, 10, either.clone());
        check_seq(ones.as_slice(), either.clone()).unwrap();
        assert!(ones.iter().all(|x| *x == 1 || *x == 2));

        assert_eq!(either.constraint().check(&3).ok().unwrap_err().len(), 1);
    }
}
