use super::*;

/// Combines two constraints so that either one may be satisfied
pub fn or<'a, A, T, S, Item>(context: S, a: A, b: T) -> OrFact<'a, A, T, Item>
where
    S: ToString,
    A: Fact<'a, Item>,
    T: Fact<'a, Item>,
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
    M1: Fact<'a, Item>,
    M2: Fact<'a, Item>,
    Item: ?Sized + Bounds<'a>,
{
    context: String,
    pub(crate) a: M1,
    pub(crate) b: M2,
    _phantom: PhantomData<&'a Item>,
}

impl<'a, P1, P2, T> Fact<'a, T> for OrFact<'a, P1, P2, T>
where
    P1: Fact<'a, T> + Fact<'a, T>,
    P2: Fact<'a, T> + Fact<'a, T>,
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
