use super::*;

/// Negates a fact
// TODO: `not` in particular would really benefit from Facts having accessible
// labels, since currently you can only get context about why a `not` fact passed,
// not why it fails.
pub fn not<'a, F, T>(fact: F) -> Fact<'a, (), T>
where
    F: 'a + Factual<'a, T>,
    T: Bounds<'a>,
{
    stateless("not", move |g, obj| {
        let label = format!("not({:?})", fact);
        let fact = fact.clone();
        brute(label, move |o| fact.clone().check(o).is_err()).mutate(g, obj)
    })
}

// /// Negates a fact, with no context given
// pub fn not<'a, F, T>(fact: F) -> Fact<'a, (), T>
// where
//     F: 'a + Factual<'a, T>,
//     T: Bounds<'a>,
// {
//     not("not", fact)
// }

#[test]
fn test_not() {
    observability::test_run().ok();
    let mut g = utils::random_generator();

    let eq1 = eq(1);
    let not1 = vec(not(eq1));

    let nums = not1.clone().build(&mut g);
    not1.clone().check(&nums).unwrap();

    assert!(nums.iter().all(|x| *x != 1));
}
