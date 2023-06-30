use super::*;

/// A constraint which is always met
pub fn always<'a, T: Target<'a>>() -> Lambda<'a, (), T> {
    lambda_unit("always", |_, t| Ok(t))
}

/// A constraint which is never met
pub fn never<'a, T: Target<'a>>(context: impl ToString) -> Lambda<'a, (), T> {
    let context = context.to_string();
    lambda_unit("never", move |g, t: T| {
        g.fail(context.clone())?;
        Ok(t)
    })
}
