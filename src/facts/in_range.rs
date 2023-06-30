use std::ops::{Bound, RangeBounds};

use super::*;

/// Specifies a range constraint
pub fn in_range<'a, R, T>(context: impl ToString, range: R) -> Lambda<'a, (), T>
where
    R: 'a + Send + Sync + RangeBounds<T> + std::fmt::Debug,
    T: Target<'a>
        + PartialOrd
        + Ord
        + num::traits::Euclid
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + num::Bounded
        + num::One,
{
    let context = context.to_string();
    lambda_unit("in_range", move |g, mut obj| {
        if !range.contains(&obj) {
            let rand = g.arbitrary(|| {
                format!(
                    "{}: expected {:?} to be contained in {:?}",
                    context, obj, range
                )
            })?;
            obj = match (range.start_bound(), range.end_bound()) {
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
                _ => panic!("Range not yet supported, sorry! {:?}", range),
            };
        }
        Ok(obj)
    })
}

#[test]
fn test_in_range() {
    observability::test_run().ok();
    let mut g = utils::random_generator();

    let positive1 = in_range("must be positive", 1..=i32::MAX);
    let positive2 = in_range("must be positive", 1..);
    let smallish = in_range("must be small in magnitude", -10..100);
    let over9000 = in_range("must be over 9000", 9001..);
    let under9000 = in_range("must be under 9000 (and no less than zero)", ..9000u32);

    let nonpositive1 = vec(not(positive1));
    let nonpositive2 = vec(not(positive2));

    let smallish_nums = smallish.clone().build(&mut g);
    let over9000_nums = over9000.clone().build(&mut g);
    let under9000_nums = under9000.clone().build(&mut g);
    let nonpositive1_nums = nonpositive1.clone().build(&mut g);
    let nonpositive2_nums = nonpositive2.clone().build(&mut g);

    dbg!(&under9000_nums);

    smallish.clone().check(&smallish_nums).unwrap();
    over9000.clone().check(&over9000_nums).unwrap();
    under9000.clone().check(&under9000_nums).unwrap();
    nonpositive1.clone().check(&nonpositive1_nums).unwrap();
    nonpositive2.clone().check(&nonpositive2_nums).unwrap();
    assert!(nonpositive1_nums.iter().all(|x| *x <= 0));
}
