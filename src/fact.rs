use arbitrary::*;
use contrafact_predicates::contrafact::Constraint;
use derive_more::From;
use std::fmt::Debug;

#[derive(From)]
pub struct FactSet<O>(Vec<Box<dyn Fact<O>>>);

impl<O> FactSet<O> {
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

pub trait Fact<O> {
    fn constraints(&mut self) -> Constraints<O>;
}

/// A set of declarative constraints. It knows how to
/// You can add to this type with `add`
pub struct Constraints<O> {
    /// Closures which run assertions on the object.
    checks: Vec<Box<dyn 'static + Fn(&mut O)>>,
    /// Closures which perform mutations on the object.
    mutations: Vec<Box<dyn 'static + Fn(&mut O, &mut Unstructured<'static>)>>,
}

impl<O> Constraints<O> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            mutations: Vec::new(),
        }
    }

    /// Add a new constraint. This generates two functions,
    /// a "check" and a "mutation", and stores them in the Constraints'
    /// internal state.
    pub fn add<T, G, C>(&mut self, get: G, constraint: C)
    where
        T: 'static + Clone + Eq + Debug + Arbitrary<'static>,
        G: 'static + Clone + Fn(&mut O) -> &mut T,
        C: 'static + Constraint<T>,
    {
        let g = get.clone();
        let p = constraint.clone();
        self.checks.push(Box::new(move |obj| {
            let t = g(obj);
            p.check(t);
        }));
        self.mutations
            .push(Box::new(move |obj, u: &mut Unstructured<'static>| {
                let t = get(obj);
                constraint.mutate(t, u)
            }));
    }

    pub fn extend(&mut self, other: Constraints<O>) {
        self.checks.extend(other.checks.into_iter());
        self.mutations.extend(other.mutations.into_iter());
    }
}

pub fn check_seq<O>(seq: &mut [O], mut facts: FactSet<O>) {
    for obj in seq {
        for f in facts.0.iter_mut() {
            f.constraints()
                .checks
                .into_iter()
                .for_each(|check| check(obj))
        }
    }
}

pub fn build_seq<O>(u: &mut Unstructured<'static>, num: usize, mut facts: FactSet<O>) -> Vec<O>
where
    O: Arbitrary<'static>,
{
    let mut seq = Vec::new();
    for i in 0..num {
        let mut obj = O::arbitrary(u).unwrap();
        for f in facts.0.iter_mut() {
            for mutate in f.constraints().mutations.into_iter() {
                mutate(&mut obj, u)
            }
        }
        seq.push(obj);
    }
    return seq;
}

mod tests {
    use super::*;
    use contrafact_predicates::prelude::*;

    #[derive(Arbitrary, Debug)]
    pub struct ChainLink {
        pub prev: u32,
        pub author: String,
    }

    pub struct ChainFact {
        prev: u32,
        author: String,
    }

    impl ChainFact {
        pub fn new(author: String) -> Self {
            Self { prev: 0, author }
        }
    }

    impl Fact<ChainLink> for ChainFact {
        fn constraints(&mut self) -> Constraints<ChainLink> {
            let mut constraints = Constraints::new();
            constraints.add(
                |o: &mut ChainLink| &mut o.author,
                predicate::eq(self.author.clone()),
            );
            constraints.add(
                |o: &mut ChainLink| &mut o.prev,
                predicate::eq(self.prev.clone()),
            );
            self.prev += 1;
            constraints
        }
    }

    #[test]
    fn test() {
        let facts = || FactSet::new(vec![Box::new(ChainFact::new("alice".into()))]);
        let mut u = Unstructured::new(&[0]);
        let mut chain = build_seq(&mut u, 10, facts());
        check_seq(chain.as_mut_slice(), facts());
        println!("Hello, world! {:?}", chain);
    }
}
