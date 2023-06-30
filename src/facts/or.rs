use super::*;

/// Combines two constraints so that either one may be satisfied
pub fn or<'a, A, T, S, Item>(context: S, a: A, b: T) -> OrFact<'a, A, T, Item>
where
    S: ToString,
    A: Factual<'a, Item>,
    T: Factual<'a, Item>,
    Item: Bounds<'a>,
{
    OrFact {
        context: context.to_string(),
        a,
        b,
        _phantom: PhantomData,
    }
}

/// Fact that combines two `Fact`s, returning the OR of the results.
///
/// This is created by the `or` function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrFact<'a, M1, M2, Item>
where
    M1: Factual<'a, Item>,
    M2: Factual<'a, Item>,
    Item: ?Sized + Bounds<'a>,
{
    context: String,
    pub(crate) a: M1,
    pub(crate) b: M2,
    _phantom: PhantomData<&'a Item>,
}

impl<'a, P1, P2, T> Factual<'a, T> for OrFact<'a, P1, P2, T>
where
    P1: Factual<'a, T> + Factual<'a, T>,
    P2: Factual<'a, T> + Factual<'a, T>,
    T: Bounds<'a>,
{
    fn mutate(&mut self, g: &mut Generator<'a>, obj: T) -> Mutation<T> {
        use rand::{thread_rng, Rng};

        let a = check_raw(&mut self.a, &obj).is_ok();
        let b = check_raw(&mut self.b, &obj).is_ok();
        match (a, b) {
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
                    self.a.mutate(g, obj)
                } else {
                    self.b.mutate(g, obj)
                }
            }
        }
    }
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
