use crate::*;

/// A Fact which applies two other facts.
pub fn and<'a, S1, S2, T>(
    a: Fact<'a, S1, T>,
    b: Fact<'a, S2, T>,
) -> Fact<'a, (Fact<'a, S1, T>, Fact<'a, S2, T>), T>
where
    T: Target<'a>,
    S1: State,
    S2: State,
{
    stateful("and", (a, b), |g, (a, b), obj| {
        let obj = a.mutate(g, obj)?;
        let obj = b.mutate(g, obj)?;
        Ok(obj)
    })
}
