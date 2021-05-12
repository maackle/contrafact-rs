//! This doesn't do anything useful, because the State cannot inform the Facts.

use crate::{fact::Bounds, Fact, Facts};

pub struct StatefulFact<'a, S, T> {
    state: S,
    update: Box<dyn Fn(&mut S)>,
    facts: Facts<'a, T>,
}

impl<'a, S, T> Fact<T> for StatefulFact<'a, S, T>
where
    T: Bounds,
{
    fn check(&mut self, obj: &T) -> crate::fact::Check {
        let result = self.facts.check(obj);
        (self.update)(&mut self.state);
        result
    }

    fn mutate(&mut self, obj: &mut T, u: &mut arbitrary::Unstructured<'static>) {
        self.facts.mutate(obj, u);
        (self.update)(&mut self.state);
    }
}
