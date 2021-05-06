use arbitrary::*;
use contrafact::*;
use std::collections::HashSet;

pub static NOISE: once_cell::sync::Lazy<Vec<u8>> = once_cell::sync::Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.gen()).take(999999).collect()
});

#[derive(Arbitrary, Debug, Clone, PartialEq, Eq, std::hash::Hash)]
enum Color {
    Cyan,
    Magenta,
    Yellow,
    Black,
}

#[derive(Arbitrary, Debug, PartialEq, Clone)]
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

impl DerivedFact<ChainLink> for ChainFact {
    fn fact(&self) -> FactBox<ChainLink> {
        let constraints: FactVec<ChainLink> = vec![
            contrafact::lens(
                "ChainLink::author",
                |o: &mut ChainLink| &mut o.author,
                predicate::eq("same author", &self.author),
            ),
            contrafact::lens(
                "ChainLink::prev",
                |o: &mut ChainLink| &mut o.prev,
                predicate::consecutive_int("increasing prev", self.prev.clone()),
            ),
            contrafact::lens(
                "ChainLink::color",
                |o: &mut ChainLink| &mut o.color,
                predicate::in_iter("valid color", &self.valid_colors),
            ),
        ];

        Box::new(constraints)
    }
}

#[test]
fn test() {
    observability::test_run().ok();
    let mut u = Unstructured::new(&NOISE);

    const NUM: u32 = 10;
    let fact = || ChainFact::new("alice".into(), &[Color::Cyan, Color::Magenta]);

    let mut chain = build_seq(&mut u, NUM as usize, fact().fact());
    dbg!(&chain);
    check_seq(chain.as_mut_slice(), fact().fact()).unwrap();

    assert!(chain.iter().all(|c| c.author == "alice"));
    assert!(chain.iter().all(|c| c.color != Color::Black));
    assert_eq!(chain.iter().last().unwrap().prev, NUM - 1);

    // there is a high probability that this will be true
    assert!(chain.iter().any(|c| c.color == Color::Magenta));
}
