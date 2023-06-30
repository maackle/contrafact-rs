use super::*;

/// A constraint which is always met
pub fn always<'a, T: Bounds<'a>>() -> StatelessFact<'a, T> {
    stateless(|_, obj| Ok(obj))
}

/// A constraint which is never met
pub fn never<'a, T: Bounds<'a>>(context: impl ToString) -> StatelessFact<'a, T> {
    let context = context.to_string();
    stateless(move |g, obj: T| {
        g.fail(context.clone())?;
        Ok(obj)
    })
}
