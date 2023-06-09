use crate::*;

/// Check that all of the constraints of all Facts are satisfied for this sequence.
/// Each Fact will run [`Fact::advance`] after each item checked, allowing stateful
/// facts to change as the sequence advances.
#[tracing::instrument(skip(fact))]
pub fn check_seq<'a, T, F>(seq: &[T], mut fact: F) -> Check
where
    F: Fact<'a, T>,
    T: Bounds<'a>,
{
    let mut reasons: Vec<String> = Vec::new();
    for (i, obj) in seq.iter().enumerate() {
        reasons.extend(
            fact.check(obj)
                .into_iter()
                .map(|reason| format!("item {}: {}", i, reason))
                .collect::<Vec<_>>(),
        );
        fact.advance(obj);
    }
    reasons.into()
}

/// Build a sequence from scratch such that all Facts are satisfied.
/// Each Fact will run [`Fact::advance`] after each item built, allowing stateful
/// facts to change as the sequence advances.
#[tracing::instrument(skip(g, fact))]
pub fn build_seq<'a, T, F>(g: &mut Generator<'a>, num: usize, mut fact: F) -> Vec<T>
where
    T: Bounds<'a>,
    F: Fact<'a, T>,
{
    let mut seq = Vec::new();
    for _i in 0..num {
        tracing::trace!("i: {}", _i);
        let obj = fact.build(g);
        fact.advance(&obj);
        seq.push(obj);
    }
    return seq;
}

/// Convenience macro for creating a collection of [`Fact`](crate::Fact)s
/// of different types.
/// Each Fact will be boxed and added to a Vec as a trait object, with their
/// types erased.
/// The resulting value also implements `Fact`.
///
/// ```
/// use contrafact::*;
///
/// let eq1 = eq_(1);
/// let not2 = not_(eq_(2));
/// let fact: FactsRef<'static, u32> = facts![eq1, not2];
/// assert!(fact.check(&1).is_ok());
/// ```
#[macro_export]
macro_rules! facts {
    ( $( $fact:expr ),+ $(,)?) => {{
        let mut fs: $crate::FactsRef<_> = Vec::new();
        $(
            fs.push(Box::new($fact));
        )+
        fs
    }};
}
