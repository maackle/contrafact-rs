use std::marker::PhantomData;

use arbitrary::Unstructured;
use lens_rs::*;

use crate::{fact::Target, Fact, *};

pub fn optical<'a, Src, Img, Optics, F, L>(
    label: L,
    optics: Optics,
    inner_fact: F,
) -> OpticalFact<'a, Src, Img, Optics, F>
where
    Src: Target<'a> + Lens<Optics, Img>,
    Img: Target<'a>,
    Optics: Clone + std::fmt::Debug,
    F: Fact<'a, Img>,
    L: ToString,
{
    OpticalFact::new(label.to_string(), optics, inner_fact)
}

/// A fact which uses a lens to apply another fact. Use [`lens1()`] to construct.
#[derive(Clone)]
pub struct OpticalFact<'a, Src, Img, Optics, F>
where
    Src: Target<'a> + Lens<Optics, Img>,
    Img: Target<'a>,
    Optics: Clone + std::fmt::Debug,
    F: Fact<'a, Img>,
{
    label: String,

    optics: Optics,

    /// The inner_fact about the inner substructure
    inner_fact: F,

    __phantom: PhantomData<&'a (Src, Img)>,
}

impl<'a, Src, Img, Optics, F> OpticalFact<'a, Src, Img, Optics, F>
where
    Src: Target<'a> + Lens<Optics, Img>,
    Img: Target<'a>,
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

impl<'a, Src, Img, Optics, F> Fact<'a, Src> for OpticalFact<'a, Src, Img, Optics, F>
where
    Src: Target<'a> + Lens<Optics, Img>,
    Img: Target<'a>,
    Optics: Clone + std::fmt::Debug,
    F: Fact<'a, Img>,
{
    // TODO: remove
    #[tracing::instrument(skip(self))]
    fn check(&mut self, t: &Src) -> Check {
        let imgs = t.traverse_ref(self.optics.clone());
        imgs.iter()
            .enumerate()
            .flat_map(|(i, img)| {
                let label = if imgs.len() > 1 {
                    format!("{}[{}]", self.label, i)
                } else {
                    self.label.clone()
                };

                self.inner_fact
                    .check(img)
                    .map(|err| format!("lens1({}){{{:?}}} > {}", label, self.optics.clone(), err))
            })
            .collect::<Vec<_>>()
            .into()
    }

    fn mutate(&mut self, mut t: Src, g: &mut Generator<'a>) -> Src {
        for img in t.traverse_mut(self.optics.clone()) {
            *img = self.inner_fact.mutate(img.clone(), g);
        }
        t
    }

    #[tracing::instrument(skip(self))]
    fn advance(&mut self, t: &Src) {
        for img in t.traverse_ref(self.optics.clone()) {
            self.inner_fact.advance(img);
        }
    }
}

#[test]
fn test_lens1() {
    let x = (1u8, (2u8, (3u8, 4u8)));

    let mut fact = OpticalFact {
        label: "".into(),
        optics: optics!(_1._1._1),
        inner_fact: eq(3),
        __phantom: PhantomData::<&((u8, (u8, (u8, u8))), u8)>,
    };

    assert_eq!(fact.check(&x).errors().len(), 1);

    fact.inner_fact = eq(4);
    assert!(fact.check(&x).is_ok());
}
