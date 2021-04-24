use arbitrary::Arbitrary;
use std::fmt::Debug;

#[derive(Clone)]
pub enum Pred<T: Clone + Eq + Debug> {
    Equals(T),
}

impl<T: Clone + Eq + Debug> Pred<T> {
    pub fn check(&self, obj: &T) {
        match self {
            Self::Equals(t) => assert_eq!(obj, t),
        }
    }

    pub fn mutate(&self, obj: &mut T) {
        match self {
            Self::Equals(t) => *obj = t.clone(),
        }
    }
}

pub trait Fact<'s, 'o, O> {
    fn define(&'s mut self, obj: &'o mut O) -> Constraints<'o, O>;
}

pub struct Constraints<'a, O> {
    checks: Vec<Box<dyn Fn(&'a mut O)>>,
    mutations: Vec<Box<dyn Fn(&'a mut O)>>,
}

impl<'a, O> Constraints<'a, O>
where
    Self: 'a,
{
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            mutations: Vec::new(),
        }
    }

    pub fn add<T, G>(&mut self, get: G, pred: Pred<T>)
    where
        T: 'static + Clone + Eq + Debug,
        G: 'static + Clone + Fn(&mut O) -> &mut T,
    {
        let g = get.clone();
        let p = pred.clone();
        self.checks.push(Box::new(move |obj| {
            let t = g(obj);
            p.check(t);
        }));
        self.mutations.push(Box::new(move |obj| {
            let t = get(obj);
            pred.mutate(t)
        }));
    }
}

// pub fn check_seq<'a, O>(seq: &[O], mut constraints: FactSet<O>)
// where
//     O: Arbitrary<'a>,
// {
//     seq.into_iter().for_each(|obj| constraints.check(obj))
// }

// pub fn build_seq<'a, O>(num: usize, mut constraints: FactSet<O>) -> Vec<O>
// where
//     O: Arbitrary<'a>,
// {
//     let mut u = Unstructured::new(&[0]);
//     itertools::unfold((), |()| {
//         let mut obj = O::arbitrary(&mut u).unwrap();
//         constraints.mutate(&mut obj);
//         Some(obj)
//     })
//     .take(num)
//     .collect()
// }

mod tests {
    use super::*;

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

    impl<'s, 'o> Fact<'s, 'o, ChainLink> for ChainFact {
        fn define(&'s mut self, obj: &'o mut ChainLink) -> Constraints<'o, ChainLink> {
            let mut constraints = Constraints::new();
            constraints.add(
                |o: &mut ChainLink| &mut o.author,
                Pred::Equals(self.author.clone()),
            );
            constraints.add(
                |o: &mut ChainLink| &mut o.prev,
                Pred::Equals(self.prev.clone()),
            );
            self.prev += 1;
            constraints
        }
    }

    #[test]
    fn test() {}
}
