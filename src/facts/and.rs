use crate::*;

/// A Fact which applies two other facts.
pub fn and<'a, T>(a: impl Factual<'a, T>, b: impl Factual<'a, T>) -> impl Factual<'a, T>
where
    T: Target<'a>,
{
    stateful("and", (a, b), |g, (a, b), obj| {
        let obj = a.mutate(g, obj)?;
        let obj = b.mutate(g, obj)?;
        Ok(obj)
    })
}
