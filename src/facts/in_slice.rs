use super::*;

/// Specifies a membership constraint
pub fn in_slice<'a, T>(context: impl ToString, slice: &'a [T]) -> LambdaUnit<'a, T>
where
    T: Target<'a> + PartialEq + Clone,
{
    let context = context.to_string();
    lambda_unit("in_slice", move |g, t| {
        Ok(if !slice.contains(&t) {
            let reason = || {
                format!(
                    "{}: expected {:?} to be contained in {:?}",
                    context, t, slice
                )
            };
            g.choose(slice, reason)?.to_owned()
        } else {
            t
        })
    })
}

/// Specifies a membership constraint
pub fn in_slice_<'a, T>(slice: &'a [T]) -> LambdaUnit<'a, T>
where
    T: Target<'a> + PartialEq + Clone,
{
    in_slice("in_slice", slice)
}
