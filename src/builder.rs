use std::{marker::PhantomData, sync::Arc};

use crate::{facts::LambdaFact, *};

pub fn build<'a, S, T, Mutator>(mutator: Mutator) -> Build<'a, S, T, Mutator>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
    Mutator: Clone + Send + Sync + FnMut(&mut Generator, &mut S, T) -> Mutation<T>,
    // Labeler: Clone + Send + Sync + Fn(String) -> String,
{
    Build {
        mutator,
        // labeler: None,
        _phantom: PhantomData,
    }
}

pub type Labeler = Arc<dyn Fn(String) -> String + Send + Sync + 'static>;

#[derive(Clone)]
pub struct Build<'a, S, T, Mutator>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
    Mutator: Clone + Send + Sync + FnMut(&mut Generator, &mut S, T) -> Mutation<T>,
    // Labeler: Clone + Send + Sync + Fn(String) -> String,
{
    mutator: Mutator,
    // labeler: Option<Labeler>,
    _phantom: PhantomData<&'a (S, T)>,
}

// impl<'a, S, T, Mutator> Fact<'a, T> for Build<'a, S, T, Mutator>
// where
//     S: Clone + Send + Sync,
//     T: Bounds<'a>,
//     Mutator: Clone + Send + Sync + FnMut(&mut Generator, &mut S, T) -> Mutation<T>,
//     // Labeler: Clone + Send + Sync + Fn(String) -> String,
// {
//     fn mutate(&mut self, g: &mut Generator<'a>, obj: T) -> Mutation<T> {
//         (self.mutator)(g, &mut self.state, obj)
//     }
// }

impl<'a, S, T, Mutator> Build<'a, S, T, Mutator>
where
    S: Clone + Send + Sync,
    T: Bounds<'a>,
    Mutator: Clone + Send + Sync + FnMut(&mut Generator, &mut S, T) -> Mutation<T>,
    // Labeler: Clone + Send + Sync + Fn(String) -> String,
{
    // pub fn label(mut self, labeler: impl Fn(String) -> String + Send + Sync + 'static) -> Self {
    //     self.labeler = Some(Arc::new(labeler));
    //     self
    // }

    // pub fn labeled(mut self, label: String) -> Self {
    //     self.labeler = Some(Arc::new(move |_| label.clone()));
    //     self
    // }

    pub fn state(&self, state: S) -> LambdaFact<'a, S, T, Mutator> {
        lambda(state, self.mutator.clone())
    }
}

// #[test]
// fn test_builder() {
//     use crate::facts::*;
//     let mut g = utils::random_generator();

//     let geom = |k|build(move |g, s, mut v| {
//         g.set(&mut v, s, "value is not geometrically increasing by 2")?;
//         *s *= 2;
//         Ok(v)
//     });

//     let from2 = geom.state(2);
//     let from3 = geom.state(3);

//     let geom2 = vec_of_length(4, from2);
//     let geom3 = vec_of_length(4, from3);

//     let list2 = geom2.clone().build(&mut g);
//     let list3 = geom3.clone().build(&mut g);
//     geom2.check(&list2).unwrap();
//     geom3.check(&list3).unwrap();
//     assert_eq!(list2, vec![2, 4, 8, 16]);
//     assert_eq!(list3, vec![3, 6, 12, 24]);
// }
