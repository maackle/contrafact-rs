use super::*;

/// Combines two constraints so that either one may be satisfied
pub fn or<'a, A, B, Item>(context: impl ToString, a: A, b: B) -> Fact<'a, (A, B), Item>
where
    A: Factual<'a, Item>,
    B: Factual<'a, Item>,
    Item: Bounds<'a>,
{
    stateful("or", (a, b), |g, (a, b), obj| {
        use rand::{thread_rng, Rng};

        let a_ok = a.clone().check(&obj).is_ok();
        let b_ok = b.clone().check(&obj).is_ok();
        match (a_ok, b_ok) {
            (true, _) => Ok(obj),
            (_, true) => Ok(obj),
            (false, false) => {
                g.fail(format!(
                    "expected either one of the following conditions to be met:
    condition 1: {:#?}
    condition 2: {:#?}",
                    a, b
                ))?;
                if thread_rng().gen::<bool>() {
                    a.mutate(g, obj)
                } else {
                    b.mutate(g, obj)
                }
            }
        }
    })
}

#[test]
fn test_or() {
    observability::test_run().ok();
    let mut g = utils::random_generator();

    let eq1 = eq("must be 1", 1);
    let eq2 = eq("must be 2", 2);
    let either = or("can be 1 or 2", eq1, eq2);

    let ones = vec(either.clone()).build(&mut g);
    vec(either.clone()).check(&ones).unwrap();
    assert!(ones.iter().all(|x| *x == 1 || *x == 2));

    assert_eq!(either.check(&3).result().unwrap().unwrap_err().len(), 1);
}
