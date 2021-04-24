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
