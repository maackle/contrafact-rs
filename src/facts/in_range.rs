use std::ops::{Bound, RangeBounds};

use super::*;

/// Specifies a range constraint
pub fn in_range<S, R, T>(context: S, range: R) -> InRangeFact<R, T>
where
    S: ToString,
    R: RangeBounds<T> + std::fmt::Debug,
    T: PartialEq
        + PartialOrd
        + Ord
        + Clone
        + std::fmt::Debug
        + num::traits::Euclid
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + num::Bounded
        + num::One,
{
    InRangeFact {
        context: context.to_string(),
        range,
        phantom: PhantomData,
    }
}

/// Specifies a range constraint
pub fn in_range_<R, T>(range: R) -> InRangeFact<R, T>
where
    R: RangeBounds<T> + std::fmt::Debug,
    T: PartialEq
        + PartialOrd
        + Ord
        + Clone
        + std::fmt::Debug
        + num::traits::Euclid
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + num::Bounded
        + num::One,
{
    in_range("in_range", range)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InRangeFact<R, T>
where
    R: RangeBounds<T> + std::fmt::Debug,
    T: PartialEq
        + PartialOrd
        + Ord
        + Clone
        + std::fmt::Debug
        + num::traits::Euclid
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + num::Bounded
        + num::One,
{
    context: String,
    range: R,
    phantom: PhantomData<T>,
}

impl<'a, R, T> Factual<'a, T> for InRangeFact<R, T>
where
    R: Send + Sync + RangeBounds<T> + std::fmt::Debug + Clone,
    T: Bounds<'a>
        + PartialEq
        + PartialOrd
        + Ord
        + Clone
        + std::fmt::Debug
        + num::traits::Euclid
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + num::Bounded
        + num::One,
{
    fn mutate(&mut self, g: &mut Generator<'a>, mut obj: T) -> Mutation<T> {
        if !self.range.contains(&obj) {
            let rand = g.arbitrary(format!(
                "{}: expected {:?} to be contained in {:?}",
                self.context, obj, self.range
            ))?;
            obj = match (self.range.start_bound(), self.range.end_bound()) {
                (Bound::Unbounded, Bound::Unbounded) => rand,
                (Bound::Included(a), Bound::Included(b)) if b.clone() - a.clone() >= T::one() => {
                    a.clone() + rand.rem_euclid(&(b.clone() - a.clone()))
                }
                (Bound::Included(a), Bound::Excluded(b)) if b.clone() - a.clone() > T::one() => {
                    a.clone() + rand.rem_euclid(&(b.clone() - a.clone()))
                }
                (Bound::Excluded(a), Bound::Included(b)) if b.clone() - a.clone() > T::one() => {
                    b.clone() - rand.rem_euclid(&(b.clone() - a.clone()))
                }
                (Bound::Unbounded, Bound::Excluded(b)) => {
                    T::min_value() + rand.rem_euclid(&(b.clone() - T::min_value()))
                }
                (Bound::Included(a), Bound::Unbounded) => {
                    a.clone() + rand.rem_euclid(&(T::max_value() - a.clone()))
                }
                _ => panic!("Range not yet supported, sorry! {:?}", self.range),
            };
        }
        Ok(obj)
    }
}
