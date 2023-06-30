use crate::*;

/// A Fact which applies two other facts.
pub fn and<'a, F1, F2, T>(a: F1, b: F2) -> Fact<'a, (F1, F2), T>
where
    T: Bounds<'a>,
    F1: Factual<'a, T>,
    F2: Factual<'a, T>,
{
    stateful((a, b), |g, (a, b), obj| {
        let obj = a.mutate(g, obj)?;
        let obj = b.mutate(g, obj)?;
        Ok(obj)
    })
}
