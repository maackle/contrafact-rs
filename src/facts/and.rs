use crate::*;

/// A Fact which applies two other facts.
pub fn and<'a, T>(a: impl Fact<'a, T>, b: impl Fact<'a, T>) -> impl Fact<'a, T>
where
    T: Target<'a>,
{
    lambda("and", (a, b), |g, (a, b), obj| {
        let obj = a.mutate(g, obj)?;
        let obj = b.mutate(g, obj)?;
        Ok(obj)
    })
}
