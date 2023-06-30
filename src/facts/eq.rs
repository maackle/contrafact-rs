use super::*;

/// Specifies an equality constraint
pub fn eq<'a, S, T>(context: S, constant: T) -> Fact<'a, (), T>
where
    S: ToString,
    T: Bounds<'a> + PartialEq + Clone,
{
    let ctx = context.to_string();
    stateless(move |g, mut obj| {
        if obj != constant {
            g.fail(format!("{}: expected {:?} == {:?}", ctx, obj, constant))?;
            obj = constant.clone();
        }
        Ok(obj)
    })
}

/// Specifies an equality constraint with no context
pub fn eq_<'a, T>(constant: T) -> Fact<'a, (), T>
where
    T: Bounds<'a> + PartialEq + Clone,
{
    eq("eq", constant)
}

/// Specifies an inequality constraint
pub fn ne<'a, S, T>(context: S, constant: T) -> Fact<'a, (), T>
where
    S: ToString,
    T: Bounds<'a> + PartialEq,
{
    not(context, eq_(constant))
}

/// Specifies an inequality constraint with no context
pub fn ne_<'a, T>(constant: T) -> Fact<'a, (), T>
where
    T: Bounds<'a> + PartialEq,
{
    ne("ne", constant)
}

#[test]
fn test_eq() {
    observability::test_run().ok();
    let mut g = utils::random_generator();

    let eq1 = vec(eq("must be 1", 1));

    let ones = eq1.clone().build(&mut g);
    eq1.clone().check(&ones).unwrap();

    assert!(ones.iter().all(|x| *x == 1));
}
