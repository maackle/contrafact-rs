use arbitrary::Arbitrary;
use contrafact::{lens, predicate, ConstraintBox, Fact};

type Id = u32;

// dhtop
#[derive(Clone, Debug, PartialEq, Arbitrary)]
enum Omega {
    AlphaBeta { id: Id, alpha: Alpha, beta: Beta },
    Alpha { id: Id, alpha: Alpha },
}

impl Omega {
    fn alpha(&mut self) -> &mut Alpha {
        match self {
            Self::AlphaBeta { alpha, .. } => alpha,
            Self::Alpha { alpha, .. } => alpha,
        }
    }

    fn id(&mut self) -> &mut Id {
        match self {
            Self::AlphaBeta { id, .. } => id,
            Self::Alpha { id, .. } => id,
        }
    }
}

// header
#[derive(Clone, Debug, PartialEq, Arbitrary)]
enum Alpha {
    Beta { id: Id, beta: Beta, data: String },
    Nil { id: Id, data: String },
}

impl Alpha {
    fn id(&mut self) -> &mut Id {
        match self {
            Self::Beta { id, .. } => id,
            Self::Nil { id, .. } => id,
        }
    }
    fn data(&mut self) -> &mut String {
        match self {
            Self::Beta { data, .. } => data,
            Self::Nil { data, .. } => data,
        }
    }
}

// entry
#[derive(Clone, Debug, PartialEq, Arbitrary)]
struct Beta {
    id: u32,
    data: String,
}

/// - All Ids should match each other. If there is a Beta, its id should match too
/// - If Omega::Alpha,     then Alpha::Nil.
/// - If Omega::AlphaBeta, then Alpha::Beta,
///     - and, the the Betas of the Alpha and the Omega should match.
/// - all data must be set as specified
struct OmegaFact {
    id: Id,
    data: String,
}

impl Fact<Omega> for OmegaFact {
    fn constraint(&mut self) -> ConstraintBox<Omega> {
        let cs = vec![
            lens(|o: &mut Omega| o.id(), predicate::eq(self.id)),
            lens(|o: &mut Omega| o.alpha().id(), predicate::eq(self.id)),
        ];

        Box::new(cs)
    }
}

//

//
//
//
//
//
//
//
//
//
//
//
//
//
//
// struct IdFact {
//     lo: u32,
//     hi: u32,
// }

// impl Fact<u32> for IdFact {
//     fn constraint(&mut self) -> ConstraintBox<u32> {
//         todo!()
//     }
// }
