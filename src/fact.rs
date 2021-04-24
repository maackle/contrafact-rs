use arbitrary::*;
use derive_more::From;
use itertools;

pub trait Fact<O> {
    fn check(&mut self, obj: &O);
    fn mutate(&mut self, obj: &mut O);
}

#[derive(From)]
pub struct FactSet<O>(Vec<Box<dyn Fact<O>>>);

impl<O> FactSet<O> {
    pub fn new(set: Vec<Box<dyn Fact<O>>>) -> Self {
        Self(set)
    }

    fn check(&mut self, obj: &O) {
        self.0.iter_mut().for_each(|c| c.check(obj))
    }

    fn mutate(&mut self, obj: &mut O) {
        self.0.iter_mut().for_each(|c| c.mutate(obj))
    }
}

pub fn check_seq<'a, O>(seq: &[O], mut constraints: FactSet<O>)
where
    O: Arbitrary<'a>,
{
    seq.into_iter().for_each(|obj| constraints.check(obj))
}

pub fn build_seq<'a, O>(num: usize, mut constraints: FactSet<O>) -> Vec<O>
where
    O: Arbitrary<'a>,
{
    let mut u = Unstructured::new(&[0]);
    itertools::unfold((), |()| {
        let mut obj = O::arbitrary(&mut u).unwrap();
        constraints.mutate(&mut obj);
        Some(obj)
    })
    .take(num)
    .collect()
}

mod tests {
    use super::*;
    #[derive(Arbitrary, Debug)]
    pub struct ChainLink {
        prev: u32,
        author: String,
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
        fn check(&mut self, obj: &ChainLink) {
            assert_eq!(obj.prev, self.prev);
            assert_eq!(obj.author, self.author);
            self.prev += 1;
        }

        fn mutate(&mut self, obj: &mut ChainLink) {
            obj.prev = self.prev.clone();
            obj.author = self.author.clone();
            self.prev += 1;
        }
    }

    #[test]
    fn test() {
        let constraints = || FactSet::new(vec![Box::new(ChainFact::new("alice".into()))]);
        let chain = build_seq(10, constraints());
        check_seq(chain.as_slice(), constraints());
        println!("Hello, world! {:?}", chain);
    }
}
