use super::*;

/// Specifies that a value should be increasing by 1 at every check/mutation
pub fn consecutive_int<S, T>(context: S, initial: T) -> ConsecutiveIntFact<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq + num::PrimInt,
{
    ConsecutiveIntFact {
        context: context.to_string(),
        counter: initial,
    }
}

/// Specifies that a value should be increasing by 1 at every check/mutation,
/// with no context given
pub fn consecutive_int_<T>(initial: T) -> ConsecutiveIntFact<T>
where
    T: std::fmt::Debug + PartialEq + num::PrimInt,
{
    consecutive_int("consecutive_int", initial)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsecutiveIntFact<T> {
    context: String,
    counter: T,
}

impl<'a, T> Fact<'a, T> for ConsecutiveIntFact<T>
where
    T: Bounds<'a> + num::PrimInt,
{
    #[tracing::instrument(fields(fact = "consecutive_int"), skip(self, g))]
    fn mutate(&mut self, g: &mut Generator<'a>, mut obj: T) -> Mutation<T> {
        if obj != self.counter {
            g.fail(&self.context)?;
            obj = self.counter.clone();
        }
        self.counter = self.counter.checked_add(&T::from(1).unwrap()).unwrap();
        Ok(obj)
    }
}
