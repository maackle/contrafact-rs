use arbitrary::*;
use derive_more::From;
use predicates::contrafact::Bounds;
use std::{fmt::Debug, marker::PhantomData, sync::Arc};

pub use predicates::contrafact::Fact;

#[macro_export]
macro_rules! facts {
    ( $( $fact:expr ,)+ ) => {{
        let mut fs: FactSet<_> = FactSet::new();
        $(
            fs.add(Box::new($fact));
        )+
        fs
    }};
}

/// A constraint defined by a custom predicate closure.
///
/// NOTE: When using during a mutation, this type can do no better than
/// brute force when finding data that matches the constraint. Therefore,
/// if the predicate is unlikely to return `true` given arbitrary data,
/// this constraint is a bad choice!
///
/// There is a fixed iteration limit, beyond which this will panic.
pub fn predicate<'a, T, F: 'a + Fn(&T) -> bool>(f: F) -> Box<PredicateFact<'a, T>> {
    Box::new(PredicateFact::new(f))
}

pub fn lens<O, T, L, F>(lens: L, fact: F) -> Box<LensFact<O, T, L, F>>
where
    T: Bounds,
    O: Bounds,
    L: 'static + Fn(&mut O) -> &mut T,
    F: 'static + Fact<T>,
{
    Box::new(LensFact::new(lens, fact))
}

pub trait FactGen<T, F>
where
    T: Bounds,
    F: Fact<T>,
{
    fn fact(&mut self) -> F;
}

/// A collection of Constraints, which can also be treated as a single Fact itself
#[derive(From)]
pub struct FactSet<O>(pub(crate) Vec<Box<dyn Fact<O>>>);

impl<O> FactSet<O>
where
    O: Bounds,
{
    /// Constructor
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, fact: Box<dyn Fact<O>>) {
        self.0.push(fact)
    }
}

impl<O> Fact<O> for FactSet<O>
where
    O: Bounds,
{
    fn check(&self, obj: &O) {
        for c in self.0.iter() {
            c.check(obj)
        }
    }

    fn mutate(&mut self, obj: &mut O, u: &mut Unstructured<'static>) {
        for c in self.0.iter_mut() {
            c.mutate(obj, u)
        }
    }
}

// impl<O, V> Fact<O> for V
// where
//     O: Bounds,
//     V: AsMut<[dyn Fact<O>]>,
// {
//     fn check(&self, obj: &O) {
//         todo!()
//     }

//     fn mutate(&mut self, obj: &mut O, u: &mut Unstructured<'static>) {
//         todo!()
//     }
// }

/// Applies a Fact to a subset of some data by means of a lens-like closure
/// which specifies the mutable subset to operate on. In other words, if type `O`
/// contains a `T`, and you have a `Fact<T>`, `LensFact` lets you lift that fact
/// into a fact about `O`.
pub struct LensFact<O, T, L, F>
where
    T: Bounds,
    O: Bounds,
    L: 'static + Fn(&mut O) -> &mut T,
    F: 'static + Fact<T>,
{
    /// Closures which run assertions on the object.
    pub(crate) lens: L,
    /// Closures which perform mutations on the object.
    pub(crate) fact: F,
    __phantom: PhantomData<(T, O)>,
}

impl<O, T, L, F> LensFact<O, T, L, F>
where
    T: Bounds,
    O: Bounds,
    L: 'static + Fn(&mut O) -> &mut T,
    F: 'static + Fact<T>,
{
    /// Constructor. Supply a lens and an existing Fact to create a new Fact.
    pub fn new(lens: L, fact: F) -> Self
    where
        T: Bounds,
        L: 'static + Fn(&mut O) -> &mut T,
        F: 'static + Fact<T>,
    {
        Self {
            lens,
            fact,
            __phantom: PhantomData,
        }
    }
}

impl<O, T, L, F> Fact<O> for LensFact<O, T, L, F>
where
    T: Bounds,
    O: Bounds,
    L: 'static + Fn(&mut O) -> &mut T,
    F: 'static + Fact<T>,
{
    fn check(&self, o: &O) {
        unsafe {
            // We can convert the immutable ref to a mutable one because `check`
            // never mutates the value, but we need `lens` to return a mutable
            // reference so it can be reused in `mutate`
            let o = o as *const O;
            let o = o as *mut O;
            self.fact.check((self.lens)(&mut *o))
        }
    }

    fn mutate(&mut self, o: &mut O, u: &mut Unstructured<'static>) {
        self.fact.mutate((self.lens)(o), u)
    }
}

/// A constraint defined by a custom predicate closure.
///
/// NOTE: When using during a mutation, this type can do no better than
/// brute force when finding data that matches the constraint. Therefore,
/// if the predicate is unlikely to return `true` given arbitrary data,
/// this constraint is a bad choice!
///
/// There is a fixed iteration limit, beyond which this will panic.
#[derive(Clone)]
pub struct PredicateFact<'a, T>(Arc<dyn 'a + Fn(&T) -> bool>);

impl<'a, T> Fact<T> for PredicateFact<'a, T>
where
    T: predicates::contrafact::Bounds,
{
    fn check(&self, t: &T) {
        assert!(self.0(t))
    }

    fn mutate(&mut self, t: &mut T, u: &mut Unstructured<'static>) {
        const ITERATION_LIMIT: usize = 100;

        for _ in 0..ITERATION_LIMIT {
            *t = T::arbitrary(u).unwrap();
            if self.0(t) {
                return;
            }
        }

        panic!(
            "Exceeded iteration limit of {} while attempting to meet a PredicateFact",
            ITERATION_LIMIT
        );
    }
}

impl<'a, T> PredicateFact<'a, T> {
    pub fn new<F: 'a + Fn(&T) -> bool>(f: F) -> Self {
        Self(Arc::new(f))
    }
}
