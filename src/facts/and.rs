use crate::*;

/// A Fact which applies two other facts.
pub fn and<'a, T>(a: impl Fact<'a, T>, b: impl Fact<'a, T>) -> impl Fact<'a, T>
where
    T: Target<'a>,
{
    lambda("and", (a, b), |g, (a, b), t| {
        let t = a.mutate(g, t)?;
        let t = b.mutate(g, t)?;
        Ok(t)
    })
}
