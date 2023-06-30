use super::*;

/// Negates a fact
// TODO: `not` in particular would really benefit from Facts having accessible
// labels, since currently you can only get context about why a `not` fact passed,
// not why it fails.
pub fn not<'a, F, S, T>(context: S, fact: F) -> NotFact<'a, F, T>
where
    S: ToString,
    F: Fact<'a, T>,
    T: Bounds<'a>,
{
    NotFact {
        context: context.to_string(),
        fact,
        _phantom: PhantomData,
    }
}

/// Negates a fact, with no context given
pub fn not_<'a, F, T>(fact: F) -> NotFact<'a, F, T>
where
    F: Fact<'a, T>,
    T: Bounds<'a>,
{
    not("not", fact)
}

#[derive(Debug, Clone)]
pub struct NotFact<'a, F, T>
where
    F: Fact<'a, T>,
    T: Bounds<'a>,
{
    context: String,
    fact: F,
    _phantom: PhantomData<&'a T>,
}

impl<'a, F, T> Fact<'a, T> for NotFact<'a, F, T>
where
    F: Fact<'a, T> + 'a,
    T: Bounds<'a>,
{
    fn mutate(&mut self, g: &mut Generator<'a>, obj: T) -> Mutation<T> {
        let label = format!("not({})", self.context.clone());
        let fact = self.fact.clone();
        brute(label, move |o| fact.clone().check(o).is_err()).mutate(g, obj)
    }
}
