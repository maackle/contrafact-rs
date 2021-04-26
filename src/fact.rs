use derive_more::From;

use crate::Constraints;

/// A collection of Facts, which can also be treated as a single Fact itself
#[derive(From)]
pub struct FactSet<O>(pub(crate) Vec<Box<dyn Fact<O>>>);

impl<O> FactSet<O> {
    /// Constructor
    pub fn new(set: Vec<Box<dyn Fact<O>>>) -> Self {
        Self(set)
    }
}

impl<O> Fact<O> for FactSet<O> {
    fn constraints(&mut self) -> Constraints<O> {
        let mut constraints = Constraints::new();
        for f in self.0.iter_mut() {
            constraints.extend(f.constraints());
        }
        constraints
    }
}

/// A stateful generator of Constraints.
///
/// A "fact" defines properties not only for a data type, but for arbitrary
/// sequences for that type, by nature of its statefulness.
pub trait Fact<O> {
    /// Generate the set of Constraints applicable given the current state.
    fn constraints(&mut self) -> Constraints<O>;
}

/// Construct a FactSet from a list of Facts with less boilerplate
#[macro_export]
macro_rules! facts {
    ( $( $fact:expr ,)+ ) => {
        FactSet::new(vec![$(Box::new($fact),)+])
    };
}

#[cfg(test)]
mod tests {}
