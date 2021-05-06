use arbitrary::*;
use contrafact::*;

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

fn chain_fact<'a>(author: &'a String, valid_colors: &'a [Color]) -> FactBox<'a, ChainLink> {
    let constraints: FactVec<ChainLink> = vec![
        contrafact::lens(
            "ChainLink::author",
            |o: &mut ChainLink| &mut o.author,
            predicate::eq("same author", author),
        ),
        contrafact::lens(
            "ChainLink::prev",
            |o: &mut ChainLink| &mut o.prev,
            predicate::consecutive_int("increasing prev", 0),
        ),
        contrafact::lens(
            "ChainLink::color",
            |o: &mut ChainLink| &mut o.color,
            predicate::in_iter("valid color", valid_colors),
        ),
    ];

    Box::new(constraints)
}

#[test]
fn test() {
    observability::test_run().ok();
    let mut u = Unstructured::new(&NOISE);

    const NUM: u32 = 10;
    let author = "alice".to_string();
    let fact = || chain_fact(&author, &[Color::Cyan, Color::Magenta]);

    let mut chain = build_seq(&mut u, NUM as usize, fact());
    dbg!(&chain);
    check_seq(chain.as_mut_slice(), fact()).unwrap();

    assert!(chain.iter().all(|c| c.author == "alice"));
    assert!(chain.iter().all(|c| c.color != Color::Black));
    assert_eq!(chain.iter().last().unwrap().prev, NUM - 1);

    // there is a high probability that this will be true
    assert!(chain.iter().any(|c| c.color == Color::Magenta));
}
