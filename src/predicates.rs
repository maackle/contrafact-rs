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
pub fn eq<S, T>(reason: S, constant: T) -> EqConstraint<T>
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
pub fn ne<S, T>(reason: S, constant: T) -> EqConstraint<T>
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
pub fn in_iter<I, S, T>(reason: S, iter: I) -> InConstraint<T>
where
    S: ToString,
    T: PartialEq + std::fmt::Debug,
    I: IntoIterator<Item = T>,
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
pub struct EqConstraint<T> {
    reason: String,
    op: EqOp,
    constant: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EqOp {
    Equal,
    NotEqual,
}

impl<T> Constraint<T> for EqConstraint<T>
where
    T: Bounds + PartialEq,
{
    fn check(&self, obj: &T) -> CheckResult {
        match self.op {
            EqOp::Equal if *obj != self.constant => vec![format!(
                "{}: expected {:?} == {:?}",
                self.reason, obj, self.constant
            )],
            EqOp::NotEqual if *obj == self.constant => vec![format!(
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
                if *obj != self.constant {
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
pub struct InConstraint<T>
where
    T: PartialEq + std::fmt::Debug,
{
    reason: String,
    inner: Vec<T>,
}

impl<T> Constraint<T> for InConstraint<T>
where
    T: Bounds,
{
    fn check(&self, obj: &T) -> CheckResult {
        if self.inner.contains(obj) {
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
        *obj = u.choose(self.inner.as_slice()).unwrap().clone();
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
    fn check(&self, _obj: &T) -> CheckResult {
        todo!()
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
    use crate::{build_seq, check_seq, NOISE};
    use arbitrary::Unstructured;

    #[test]
    fn test_eq() {
        observability::test_run().ok();

        let mut u = Unstructured::new(&NOISE);

        let mustbe1 = eq("must be 1", 1).to_fact();

        let ones = build_seq(&mut u, 3, mustbe1.clone());
        check_seq(ones.as_slice(), mustbe1.clone()).unwrap();

        assert!(ones.iter().all(|x| *x == 1));
    }
}
