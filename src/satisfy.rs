use crate::*;

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
