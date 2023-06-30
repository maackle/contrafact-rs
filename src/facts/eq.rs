use super::*;

/// Specifies an equality constraint
pub fn eq<'a, T>(constant: T) -> Lambda<'a, (), T>
where
    T: Target<'a> + PartialEq + Clone,
{
    let label = format!("eq({:?})", constant);
    lambda_unit(label, move |g, mut t| {
        if t != constant {
            g.fail(format!("expected {:?} == {:?}", t, constant))?;
            t = constant.clone();
        }
        Ok(t)
    })
}

/// Specifies an inequality constraint
pub fn ne<'a, S, T>(constant: T) -> Lambda<'a, (), T>
where
    S: ToString,
    T: Target<'a> + PartialEq,
{
    not(eq(constant)).labeled("ne")
}

#[test]
fn test_eq() {
    observability::test_run().ok();
    let mut g = utils::random_generator();

    let eq1 = vec(eq(1));

    let ones = eq1.clone().build(&mut g);
    eq1.clone().check(&ones).unwrap();

    assert!(ones.iter().all(|x| *x == 1));
}
