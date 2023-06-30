use super::{brute::brute_labeled, lambda::LambdaFact, *};

/// Negates a fact
// TODO: `not` in particular would really benefit from Facts having accessible
// labels, since currently you can only get context about why a `not` fact passed,
// not why it fails.
pub fn not<'a, F, T>(context: impl ToString, fact: F) -> LambdaFact<'a, (), T>
where
    F: 'a + Fact<'a, T>,
    T: Bounds<'a>,
{
    let context = context.to_string();
    lambda_unit(move |g, obj| {
        let label = format!("not({})", context.clone());
        let fact = fact.clone();
        brute(label, move |o| fact.clone().check(o).is_err()).mutate(g, obj)
    })
}

/// Negates a fact, with no context given
pub fn not_<'a, F, T>(fact: F) -> LambdaFact<'a, (), T>
where
    F: 'a + Fact<'a, T>,
    T: Bounds<'a>,
{
    not("not", fact)
}
