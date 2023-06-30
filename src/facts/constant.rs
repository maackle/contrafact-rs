use super::*;

/// A constraint which is always met
pub fn always() -> ConstantFact {
    ConstantFact(true, "always".to_string())
}

/// A constraint which is never met
pub fn never<S: ToString>(context: S) -> ConstantFact {
    ConstantFact(false, context.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantFact(bool, String);

impl<'a, T> Factual<'a, T> for ConstantFact
where
    T: Bounds<'a> + PartialEq + Clone,
{
    #[tracing::instrument(fields(fact = "bool"), skip(self, g))]
    fn mutate(&mut self, g: &mut Generator<'_>, obj: T) -> Mutation<T> {
        if !self.0 {
            g.fail("never() encountered.")?;
        }
        Ok(obj)
    }
}
