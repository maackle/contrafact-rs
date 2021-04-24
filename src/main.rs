use arbitrary::Arbitrary;

mod fact;
mod fact2;
mod lens;
mod playground;

pub use fact::*;
pub use lens::*;

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

fn main() {
    let constraints = || FactSet::new(vec![Box::new(ChainFact::new("alice".into()))]);
    let chain = build_seq(10, constraints());
    check_seq(chain.as_slice(), constraints());
    println!("Hello, world! {:?}", chain);
}
