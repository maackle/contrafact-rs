use crate::{
    fact::{Bounds, CheckResult},
    Fact,
};
use arbitrary::Unstructured;

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