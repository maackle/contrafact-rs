use super::*;

/// Specifies a membership constraint
pub fn in_slice<'a, T>(context: impl ToString, slice: &'a [T]) -> StatelessFact<'a, T>
where
    T: Bounds<'a> + PartialEq + Clone,
{
    let context = context.to_string();
    stateless("in_slice", move |g, obj| {
        Ok(if !slice.contains(&obj) {
            let reason = || {
                format!(
                    "{}: expected {:?} to be contained in {:?}",
                    context, obj, slice
                )
            };
            g.choose(slice, reason)?.to_owned()
        } else {
            obj
        })
    })
}

/// Specifies a membership constraint
pub fn in_slice_<'a, T>(slice: &'a [T]) -> StatelessFact<'a, T>
where
    T: Bounds<'a> + PartialEq + Clone,
{
    in_slice("in_slice", slice)
}
