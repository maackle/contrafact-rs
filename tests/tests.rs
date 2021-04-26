use arbitrary::*;
use contrafact::*;
use std::collections::HashSet;

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
        let mut constraints = Constraints::<ChainLink>::new();
        constraints.add(|o| &mut o.author, constraint::eq(self.author.clone()));
        constraints.add(|o| &mut o.prev, constraint::eq(self.prev.clone()));
        constraints.add(
            |o| &mut o.color,
            constraint::in_iter(self.valid_colors.clone()),
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
