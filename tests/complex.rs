use arbitrary::Arbitrary;
use contrafact::{facts::*, *};

use either::Either;
#[cfg(feature = "optics")]
use lens_rs::{optics, Lens, Prism};

type Id = u32;

// Similar to Holochain's DhtOp
#[derive(Clone, Debug, PartialEq, Arbitrary)]
enum Omega {
    AlphaBeta { id: Id, alpha: Alpha, beta: Beta },
    Alpha { id: Id, alpha: Alpha },
}

#[allow(unused)]
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

    fn pi(&self) -> Pi {
        match self.clone() {
            Self::AlphaBeta { alpha, beta, .. } => Pi(alpha, Some(beta)),
            Self::Alpha { alpha, .. } => Pi(alpha, None),
        }
    }

    fn id(&self) -> &Id {
        match self {
            Self::AlphaBeta { id, .. } => id,
            Self::Alpha { id, .. } => id,
        }
    }

    fn id_mut(&mut self) -> &mut Id {
        match self {
            Self::AlphaBeta { id, .. } => id,
            Self::Alpha { id, .. } => id,
        }
    }
}

// Similar to Holochain's Action
#[derive(Clone, Debug, PartialEq, Arbitrary)]
enum Alpha {
    Beta { id: Id, beta: Beta, data: String },
    Nil { id: Id, data: String },
}

#[allow(unused)]
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

// Similar to Holochain's Entry
#[derive(Clone, Debug, PartialEq, Arbitrary)]
struct Beta {
    id: u32,
    data: String,
}

#[derive(Clone, Debug, PartialEq, Arbitrary)]
/// Similar to Holochain's SignedActionHashed
struct Sigma {
    alpha: Alpha,
    id2: Id,
    sig: String,
}

#[derive(Clone, Debug, PartialEq, Arbitrary)]
#[cfg_attr(feature = "optics", derive(Lens))]
/// Similar to Holochain's Record
struct Rho {
    #[cfg_attr(feature = "optics", optic)]
    sigma: Sigma,
    #[cfg_attr(feature = "optics", optic)]
    beta: Option<Beta>,
}

/// Some struct needed to set the values of a Sigma whenever its Alpha changes.
/// Analogous to Holochain's Keystore (MetaLairClient).
struct AlphaSigner;

impl AlphaSigner {
    fn sign(&self, mut alpha: Alpha) -> Sigma {
        Sigma {
            id2: alpha.id().clone() * 2,
            sig: alpha.id().to_string(),
            alpha,
        }
    }
}

#[allow(unused)]
fn alpha_fact() -> impl Factual<'static, Alpha> {
    facts![lens("Alpha::id", |a: &mut Alpha| a.id(), id_fact(None))]
}

fn beta_fact() -> impl Factual<'static, Beta> {
    facts![lens("Beta::id", |a: &mut Beta| &mut a.id, id_fact(None))]
}

/// Just a pair of an Alpha with optional Beta.
/// An intermediate type not used "in production" but useful for writing Fact 'static, against
#[derive(Clone, Debug, PartialEq, Arbitrary)]
struct Pi(Alpha, Option<Beta>);

fn pi_beta_match() -> impl Factual<'static, Pi> {
    facts![brute(
        "Pi alpha has matching beta iff beta is Some",
        |p: &Pi| match p {
            Pi(Alpha::Beta { beta, .. }, Some(b)) => beta == b,
            Pi(Alpha::Nil { .. }, None) => true,
            _ => false,
        }
    )]
}

fn id_fact(id: Option<Id>) -> impl Factual<'static, Id> {
    let le = brute("< u32::MAX", |id: &Id| *id < Id::MAX / 2);

    if let Some(id) = id {
        Either::Left(facts![le, eq(id)])
    } else {
        Either::Right(facts![le])
    }
}

/// - id must be set as specified
/// - All Ids should match each other. If there is a Beta, its id should match too.
fn pi_fact(id: Id) -> impl Factual<'static, Pi> {
    let alpha_fact = facts![
        lens("Alpha::id", |a: &mut Alpha| a.id(), id_fact(Some(id))),
        // lens("Alpha::data", |a: &mut Alpha| a.data(), eq(data)),
    ];
    let beta_fact = lens("Beta::id", |b: &mut Beta| &mut b.id, id_fact(Some(id)));
    facts![
        pi_beta_match(),
        lens("Pi::alpha", |o: &mut Pi| &mut o.0, alpha_fact),
        prism("Pi::beta", |o: &mut Pi| o.1.as_mut(), beta_fact),
    ]
}

/// - All Ids should match each other. If there is a Beta, its id should match too
/// - If Omega::Alpha,     then Alpha::Nil.
/// - If Omega::AlphaBeta, then Alpha::Beta,
///     - and, the the Betas of the Alpha and the Omega should match.
/// - all data must be set as specified
fn omega_fact(id: Id) -> impl Factual<'static, Omega> {
    let omega_pi = LensFact::new(
        "Omega -> Pi",
        |o| match o {
            Omega::AlphaBeta { alpha, beta, .. } => Pi(alpha, Some(beta)),
            Omega::Alpha { alpha, .. } => Pi(alpha, None),
        },
        |o, pi| {
            let id = o.id().clone();
            match pi {
                Pi(alpha, Some(beta)) => Omega::AlphaBeta { id, alpha, beta },
                Pi(alpha, None) => Omega::Alpha { id, alpha },
            }
        },
        pi_fact(id),
    );

    facts![
        omega_pi,
        lens("Omega::id", |o: &mut Omega| o.id_mut(), id_fact(Some(id))),
    ]
}

#[allow(unused)]
fn sigma_fact() -> impl Factual<'static, Sigma> {
    let id2_fact = LensFact::new(
        "Sigma::id is correct",
        |mut s: Sigma| (s.id2, *(s.alpha.id()) * 2),
        |mut s, (_, id2)| {
            s.id2 = id2;
            s
        },
        same(),
    );
    let sig_fact = LensFact::new(
        "Sigma::sig is correct",
        |mut s: Sigma| (s.sig, s.alpha.id().to_string()),
        |mut s, (_, sig)| {
            s.sig = sig;
            s
        },
        same(),
    );
    facts![
        lens("Sigma::id", |o: &mut Sigma| o.alpha.id(), id_fact(None)),
        id2_fact
    ]
}

/// The inner Sigma is correct wrt to signature
/// XXX: this is a little wonky, probably room for improvement.
fn rho_fact(id: Id, signer: AlphaSigner) -> impl Factual<'static, Rho> {
    let rho_pi = LensFact::new(
        "Rho -> Pi",
        |rho: Rho| Pi(rho.sigma.alpha, rho.beta),
        move |mut rho, Pi(a, b)| {
            rho.sigma = signer.sign(a);
            rho.beta = b;
            rho
        },
        pi_fact(id),
    );

    #[cfg(not(feature = "optics"))]
    {
        facts![
            lens("Rho -> Sigma", |rho: &mut Rho| &mut rho.sigma, sigma_fact()),
            rho_pi
        ]
    }

    #[cfg(feature = "optics")]
    {
        facts![
            optical("Rho -> Sigma", optics!(sigma), sigma_fact()),
            rho_pi
        ]
    }
}

#[test]
fn test_rho_fact() {
    observability::test_run().ok();
    let mut g = utils::random_generator();

    let fact = rho_fact(5, AlphaSigner);
    let mut rho = fact.clone().build(&mut g);
    assert!(fact.clone().check(&rho).is_ok());
    assert_eq!(rho.sigma.id2, 10);
    assert_eq!(rho.sigma.sig, "5".to_string());

    rho.sigma.id2 = 9;
    assert!(fact.clone().check(&rho).is_err());

    dbg!(rho);
}

#[test]
fn test_omega_fact() {
    observability::test_run().ok();
    let mut g = utils::random_generator();

    let mut fact = omega_fact(11);

    let beta = beta_fact().build(&mut g);

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

    valid1 = fact.mutate(&mut g, valid1).unwrap();
    fact.clone().check(dbg!(&valid1)).unwrap();

    valid2 = fact.mutate(&mut g, valid2).unwrap();
    fact.clone().check(dbg!(&valid2)).unwrap();

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
    assert!(
        dbg!(fact
            .clone()
            .check(dbg!(&invalid1))
            .result()
            .unwrap()
            .unwrap_err())
        .len()
            > 0
    );
    invalid1 = fact.mutate(&mut g, invalid1).unwrap();
    fact.clone().check(dbg!(&invalid1)).unwrap();

    // Ensure that check fails for invalid data
    assert!(
        dbg!(fact
            .clone()
            .check(dbg!(&invalid2))
            .result()
            .unwrap()
            .unwrap_err())
        .len()
            > 0
    );
    invalid2 = fact.mutate(&mut g, invalid2).unwrap();
    fact.clone().check(dbg!(&invalid2)).unwrap();
}
