use std::marker::PhantomData;

use arbitrary::Unstructured;
use lens_rs::*;

use crate::{fact::Bounds, Fact, *};

pub fn lens<'a, Src, Img, Optics, F, L>(
    label: L,
    optics: Optics,
    inner_fact: F,
) -> LensFact<'a, Src, Img, Optics, F>
where
    Src: Bounds<'a> + Lens<Optics, Img>,
    Img: Bounds<'a> + Clone,
    Optics: Clone + std::fmt::Debug,
    F: Fact<'a, Img>,
    L: ToString,
{
    LensFact::new(label.to_string(), optics, inner_fact)
}

/// A fact which uses a lens to apply another fact. Use [`lens()`] to construct.
#[derive(Clone)]
pub struct LensFact<'a, Src, Img, Optics, F>
where
    Src: Bounds<'a> + Lens<Optics, Img>,
    Img: Bounds<'a> + Clone,
    Optics: Clone + std::fmt::Debug,
    F: Fact<'a, Img>,
{
    label: String,

    optics: Optics,

    /// The inner_fact about the inner substructure
    inner_fact: F,

    __phantom: PhantomData<&'a (Src, Img)>,
}

impl<'a, Src, Img, Optics, F> LensFact<'a, Src, Img, Optics, F>
where
    Src: Bounds<'a> + Lens<Optics, Img>,
    Img: Bounds<'a> + Clone,
    Optics: Clone + std::fmt::Debug,
    F: Fact<'a, Img>,
{
    pub fn new(label: String, optics: Optics, inner_fact: F) -> Self {
        Self {
            label,
            optics,
            inner_fact,
            __phantom: PhantomData::<&(Src, Img)>,
        }
    }
}

impl<'a, Src, Img, Optics, F> Fact<'a, Src> for LensFact<'a, Src, Img, Optics, F>
where
    Src: Bounds<'a> + Lens<Optics, Img>,
    Img: Bounds<'a> + Clone,
    Optics: Clone + std::fmt::Debug,
    F: Fact<'a, Img>,
{
    #[tracing::instrument(skip(self))]
    fn check(&self, obj: &Src) -> Check {
        self.inner_fact
            .check(obj.view_ref(self.optics.clone()))
            .map(|err| format!("lens({:?}) > {}", self.optics.clone(), err))
    }

    #[tracing::instrument(skip(self, u))]
    fn mutate(&self, mut obj: Src, u: &mut Unstructured<'a>) -> Src {
        let t = obj.view_ref(self.optics.clone());
        let t = self.inner_fact.mutate(t.clone(), u);
        *obj.view_mut(self.optics.clone()) = t;
        obj
    }

    #[tracing::instrument(skip(self))]
    fn advance(&mut self, obj: &Src) {
        self.inner_fact.advance(obj.view_ref(self.optics.clone()))
    }
}

#[test]
fn test_lens() {
    let x = (1u8, (2u8, (3u8, 4u8)));

    let mut fact = LensFact {
        label: "".into(),
        optics: optics!(_1._1._1),
        inner_fact: eq_(3),
        __phantom: PhantomData::<&((u8, (u8, (u8, u8))), u8)>,
    };

    assert_eq!(fact.check(&x).errors().len(), 1);

    fact.inner_fact = eq_(4);
    assert!(fact.check(&x).is_ok());
}
