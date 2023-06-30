/// Convenience macro for creating a collection of [`Fact`](crate::Fact)s
/// of different types.
/// The Facts will be composed into a nested series of [`AndFact`] which causes
/// all facts to be applied in sequence. The collection of Facts is also a Fact.
///
/// ```
/// use contrafact::*;
///
/// let eq1 = eq_(1);
/// let not2 = not_(eq_(2));
/// let mut fact = facts![eq1, not2];
/// assert!(fact.check(&1).is_ok());
/// ```
#[macro_export]
macro_rules! facts {

    ( $fact:expr $(,)?) => { $fact };

    ( $fact_0:expr, $fact_1:expr $( , $fact_n:expr )* $(,)? ) => {{
        facts![
            $crate::AndFact::new($fact_0, $fact_1),
            $( $fact_n , )*
        ]
    }};
}
