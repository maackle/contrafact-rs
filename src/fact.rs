use arbitrary::*;
use derive_more::From;
use predicates::contrafact::Constraint;
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
        let constraint2 = constraint.clone();
        self.checks.push(Box::new(move |obj| {
            let t = g(obj);
            constraint.check(t);
        }));
        self.mutations
            .push(Box::new(move |obj, u: &mut Unstructured<'static>| {
                let t = get(obj);
                constraint2.mutate(t, u)
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
    for _i in 0..num {
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

#[macro_export]
macro_rules! facts {
    ( $( $fact:expr ,)+ ) => {
        FactSet::new(vec![$(Box::new($fact),)+])
    };
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use predicates::prelude::*;

    static NOISE: once_cell::sync::Lazy<Vec<u8>> =
        once_cell::sync::Lazy::new(|| bring_on_the_noise(99999));

    #[derive(Arbitrary, Debug, Clone, PartialEq, Eq, std::hash::Hash)]
    enum Color {
        Cyan,
        Magenta,
        Yellow,
        Black,
    }

    #[derive(Arbitrary, Debug)]
    struct ChainLink {
        prev: u32,
        author: String,
        color: Color,
    }

    struct ChainFact {
        prev: u32,
        author: String,
        valid_colors: HashSet<Color>,
    }

    impl ChainFact {
        fn new(author: String, valid_colors: &[Color]) -> Self {
            Self {
                prev: 0,
                author,
                valid_colors: valid_colors.into_iter().cloned().collect(),
            }
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
            constraints.add(
                |o: &mut ChainLink| &mut o.color,
                predicate::in_iter(self.valid_colors.clone()),
            );
            self.prev += 1;
            constraints
        }
    }

    pub fn bring_on_the_noise(size: usize) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        std::iter::repeat_with(|| rng.gen()).take(size).collect()
    }

    #[test]
    fn test() {
        const NUM: u32 = 10;
        let facts = || {
            facts![ChainFact::new(
                "alice".into(),
                &[Color::Cyan, Color::Magenta],
            ),]
        };
        let mut u = Unstructured::new(&NOISE);

        let mut chain = build_seq(&mut u, NUM as usize, facts());
        check_seq(chain.as_mut_slice(), facts());

        dbg!(&chain);

        assert!(chain.iter().all(|c| c.author == "alice"));
        assert!(chain.iter().all(|c| c.color != Color::Black));
        assert_eq!(chain.iter().last().unwrap().prev, NUM - 1);

        // there is a high probability that this will be true
        assert!(chain.iter().any(|c| c.color == Color::Magenta));
    }
}
