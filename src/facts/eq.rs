use std::fmt::Display;

use super::*;

/// Specifies an equality constraint
pub fn eq<'a, T>(constant: T) -> Fact<'a, (), T>
where
    T: Bounds<'a> + PartialEq + Clone + Display,
{
    let label = format!("eq({})", constant);
    stateless(label, move |g, mut obj| {
        if obj != constant {
            g.fail(format!("expected {:?} == {:?}", obj, constant))?;
            obj = constant.clone();
        }
        Ok(obj)
    })
}

/// Specifies an inequality constraint
pub fn ne<'a, S, T>(constant: T) -> Fact<'a, (), T>
where
    S: ToString,
    T: Bounds<'a> + PartialEq + Display,
{
    not(eq(constant)).label("ne")
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
