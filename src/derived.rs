use crate::{
    fact::{Bounds, CheckResult, FactBox},
    Fact,
};
use arbitrary::Unstructured;

/// A "DerivedFact" is simply a data type which can produce a `Fact`.
/// When producing the constraint, there is an opportunity to mutate the DerivedFact's
/// state, causing it to produce a different Fact next time `.constraint`
/// is called. This allows for `fold()`-like application of a DerivedFact to a sequence
/// of data.
pub trait DerivedFact<T> {
    /// Produce the constraint given the current state of this DerivedFact
    fn fact(&self) -> FactBox<'_, T>;
}

impl<T, F> DerivedFact<T> for Vec<F>
where
    T: 'static + Bounds,
    F: DerivedFact<T>,
{
    fn fact(&self) -> FactBox<'_, T> {
        Box::new(self.iter().map(|f| f.fact()).collect::<Vec<_>>())
    }
}

/// Check that all of the constraints of all Facts are satisfied for this sequence.
#[tracing::instrument(skip(fact))]
pub fn check_seq<T, F>(seq: &[T], mut fact: F) -> CheckResult
where
    F: Fact<T>,
    T: Bounds,
{
    let mut reasons: Vec<String> = Vec::new();
    for (i, obj) in seq.iter().enumerate() {
        reasons.extend(
            fact.check(obj)
                .into_iter()
                .map(|reason| format!("item {}: {}", i, reason))
                .collect::<Vec<_>>(),
        );
    }
    reasons.into()
}

/// Build a sequence from scratch such that all Facts are satisfied.
#[tracing::instrument(skip(u, fact))]
pub fn build_seq<T, F>(u: &mut Unstructured<'static>, num: usize, mut fact: F) -> Vec<T>
where
    T: Bounds,
    F: Fact<T>,
{
    let mut seq = Vec::new();
    for _i in 0..num {
        let mut obj = T::arbitrary(u).unwrap();
        tracing::trace!("i: {}", _i);
        fact.mutate(&mut obj, u);
        seq.push(obj);
    }
    return seq;
}

/// Convenience macro for creating a collection of `Facts`s of different types.
/// The resulting value also implements `DerivedFact`.
#[macro_export]
macro_rules! facts {
    ( $( $fact:expr ,)+ ) => {{
        let mut fs = Vec::new();
        $(
            fs.push(Box::new($fact));
        )+
        fs
    }};
}
