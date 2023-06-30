use super::{brute::brute_labeled, *};

/// Specifies an equality constraint between two items in a tuple
pub fn same<'a, T>() -> LambdaUnit<'a, (T, T)>
where
    T: Target<'a> + PartialEq,
{
    lambda_unit("same", |g, mut t: (T, T)| {
        let o = t.clone();
        let reason = move || format!("must be same: expected {:?} == {:?}", o.0.clone(), o.1);
        g.set(&mut t.0, &t.1, reason)?;
        Ok(t)
    })
}

/// Specifies an inequality constraint between two items in a tuple
pub fn different<'a, T>() -> LambdaUnit<'a, (T, T)>
where
    T: Target<'a> + PartialEq,
{
    brute_labeled(|(a, b)| {
        if a == b {
            Ok(Err(format!(
                "must be different: expected {:?} != {:?}",
                a, b
            )))
        } else {
            Ok(Ok(()))
        }
    })
}

#[test]
fn test_same() {
    observability::test_run().ok();
    let mut g = utils::random_generator();

    {
        let f = vec(same::<u8>());
        let nums = f.clone().build(&mut g);
        f.clone().check(&nums).unwrap();
        assert!(nums.iter().all(|(a, b)| a == b));
    }
    {
        let f = vec(different::<u8>());
        let nums = f.clone().build(&mut g);
        f.clone().check(&nums).unwrap();
        assert!(nums.iter().all(|(a, b)| a != b));
    }
}
