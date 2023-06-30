use super::*;

/// Specifies that a value should be increasing by 1 at every check/mutation
pub fn consecutive_int<'a, S>(context: impl ToString, initial: S) -> Lambda<'a, S, S>
where
    S: Target<'a> + std::fmt::Debug + PartialEq + num::PrimInt,
{
    let context = context.to_string();
    lambda("consecutive_int", initial, move |g, counter, mut t| {
        if t != *counter {
            g.fail(&context)?;
            t = counter.clone();
        }
        *counter = counter.checked_add(&S::from(1).unwrap()).unwrap();
        Ok(t)
    })
}

/// Specifies that a value should be increasing by 1 at every check/mutation,
/// with no context given
pub fn consecutive_int_<'a, T>(initial: T) -> Lambda<'a, T, T>
where
    T: Target<'a> + PartialEq + num::PrimInt,
{
    consecutive_int("consecutive_int", initial)
}
