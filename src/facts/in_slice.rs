use super::*;

/// Specifies a membership constraint
pub fn in_slice<'a, S, T>(context: S, slice: &'a [T]) -> InSliceFact<'a, T>
where
    S: ToString,
    T: 'a + PartialEq + std::fmt::Debug + Clone,
{
    InSliceFact {
        context: context.to_string(),
        slice,
    }
}

/// Specifies a membership constraint
pub fn in_slice_<'a, T>(slice: &'a [T]) -> InSliceFact<'a, T>
where
    T: 'a + PartialEq + std::fmt::Debug + Clone,
{
    in_slice("in_slice", slice)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InSliceFact<'a, T>
where
    T: 'a + PartialEq + std::fmt::Debug + Clone,
    // I: Iterator<Item = &'a T>,
{
    context: String,
    slice: &'a [T],
}

impl<'a, 'b: 'a, T> Factual<'a, T> for InSliceFact<'b, T>
where
    T: 'b + Bounds<'a> + Clone,
    // I: Iterator<Item = &'b T>,
{
    fn mutate(&mut self, g: &mut Generator<'a>, obj: T) -> Mutation<T> {
        Ok(if !self.slice.contains(&obj) {
            g.choose(
                self.slice,
                format!(
                    "{}: expected {:?} to be contained in {:?}",
                    self.context, obj, self.slice
                ),
            )?
            .to_owned()
        } else {
            obj
        })
    }
}
