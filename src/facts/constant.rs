use super::*;

/// A constraint which is always met
pub fn always<'a, T: Target<'a>>() -> StatelessFact<'a, T> {
    stateless("always", |_, obj| Ok(obj))
}

/// A constraint which is never met
pub fn never<'a, T: Target<'a>>(context: impl ToString) -> StatelessFact<'a, T> {
    let context = context.to_string();
    stateless("never", move |g, obj: T| {
        g.fail(context.clone())?;
        Ok(obj)
    })
}
