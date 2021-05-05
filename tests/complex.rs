use arbitrary::{Arbitrary, Unstructured};
use contrafact::{constraints, custom, lens, predicate, prism, ConstraintBox, Fact};

pub static NOISE: once_cell::sync::Lazy<Vec<u8>> = once_cell::sync::Lazy::new(|| {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    std::iter::repeat_with(|| rng.gen()).take(999999).collect()
});

type Id = u32;

// Similar to Holochain DhtOp
#[derive(Clone, Debug, PartialEq, Arbitrary)]
enum Omega {
    AlphaBeta { id: Id, alpha: Alpha, beta: Beta },
    Alpha { id: Id, alpha: Alpha },
}

impl Omega {
    fn alpha(&self) -> &Alpha {
        match self {
            Self::AlphaBeta { alpha, .. } => alpha,
            Self::Alpha { alpha, .. } => alpha,
        }
    }

    fn alpha_mut(&mut self) -> &mut Alpha {
        match self {
            Self::AlphaBeta { alpha, .. } => alpha,
            Self::Alpha { alpha, .. } => alpha,
        }
    }

    fn _beta(&self) -> Option<&Beta> {
        match self {
            Self::AlphaBeta { beta, .. } => Some(beta),
            Self::Alpha { .. } => None,
        }
    }

    fn beta_mut(&mut self) -> Option<&mut Beta> {
        match self {
            Self::AlphaBeta { beta, .. } => Some(beta),
            Self::Alpha { .. } => None,
        }
    }

    fn id(&mut self) -> &mut Id {
        match self {
            Self::AlphaBeta { id, .. } => id,
            Self::Alpha { id, .. } => id,
        }
    }
}

// Similar to Holochain Header
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

// Similar to Holochain Entry
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
    fn constraint(&self) -> ConstraintBox<Omega> {
        let alpha_constraint = constraints![
            lens(
                "Alpha::id",
                |a: &mut Alpha| a.id(),
                predicate::eq("id", &self.id)
            ),
            lens(
                "Alpha::data",
                |a: &mut Alpha| a.data(),
                predicate::eq("data", &self.data)
            ),
        ];
        let beta_constraint = lens(
            "Beta::id",
            |b: &mut Beta| &mut b.id,
            predicate::eq("id", &self.id),
        );
        let omega_constraint = constraints![
            custom("Omega variant matches Alpha variant", |o: &Omega| {
                match (o, o.alpha()) {
                    (Omega::AlphaBeta { .. }, Alpha::Beta { .. }) => true,
                    (Omega::Alpha { .. }, Alpha::Nil { .. }) => true,
                    _ => false,
                }
            }),
            lens(
                "Omega::id",
                |o: &mut Omega| o.id(),
                predicate::eq("id", &self.id)
            ),
        ];

        constraints![
            omega_constraint,
            lens(
                "Omega::alpha",
                |o: &mut Omega| o.alpha_mut(),
                alpha_constraint
            ),
            prism("Omega::beta", |o: &mut Omega| o.beta_mut(), beta_constraint),
        ]
    }
}

#[test]
fn test_omega_fact() {
    observability::test_run().ok();
    let mut u = Unstructured::new(&NOISE);

    let fact = OmegaFact {
        id: 11,
        data: "spartacus".into(),
    };

    let beta = Beta::arbitrary(&mut u).unwrap();

    let mut valid1 = Omega::Alpha {
        id: 8,
        alpha: Alpha::Nil {
            id: 3,
            data: "cheese".into(),
        },
    };

    let mut valid2 = Omega::AlphaBeta {
        id: 8,
        alpha: Alpha::Nil {
            id: 3,
            data: "cheese".into(),
        },
        beta: beta.clone(),
    };

    fact.constraint().mutate(&mut valid1, &mut u);
    fact.constraint().check(dbg!(&valid1)).unwrap();

    fact.constraint().mutate(&mut valid2, &mut u);
    fact.constraint().check(dbg!(&valid2)).unwrap();

    let mut invalid1 = Omega::Alpha {
        id: 8,
        alpha: Alpha::Beta {
            id: 3,
            data: "cheese".into(),
            beta: beta.clone(),
        },
    };

    let mut invalid2 = Omega::AlphaBeta {
        id: 8,
        alpha: Alpha::Nil {
            id: 3,
            data: "cheese".into(),
        },
        beta: beta.clone(),
    };

    // Ensure that check fails for invalid data
    assert_eq!(
        dbg!(fact.constraint().check(dbg!(&invalid1)).ok().unwrap_err()).len(),
        4,
    );
    fact.constraint().mutate(&mut invalid1, &mut u);
    fact.constraint().check(dbg!(&invalid1)).unwrap();

    // Ensure that check fails for invalid data
    assert_eq!(
        dbg!(fact.constraint().check(dbg!(&invalid2)).ok().unwrap_err()).len(),
        5,
    );
    fact.constraint().mutate(&mut invalid2, &mut u);
    fact.constraint().check(dbg!(&invalid2)).unwrap();
}
