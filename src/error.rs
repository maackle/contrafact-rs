pub type CheckMsg = String;

#[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
pub enum CheckError {
    Check(CheckMsg),
    Internal(ContrafactError),
}

#[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
pub enum ContrafactError {
    // TODO: uncomment if this PR is merged:
    // https://github.com/rust-fuzz/arbitrary/pull/153
    // UnexpectedError(arbitrary::Error),
    Other(String),
}

pub type ContrafactResult<T> = Result<T, ContrafactError>;

#[derive(Clone, Debug, derive_more::From)]
pub enum MutationError {
    Check(CheckMsg),
    Arbitrary(arbitrary::Error),
    Internal(ContrafactError),
}

pub type Mutation<T> = Result<T, MutationError>;

pub trait MutationExt<T> {
    // fn checks_ok(self) -> Result<Result<T, CheckError>, MutationError>;
    fn map_check_err(self, f: impl Fn(CheckMsg) -> CheckMsg) -> Mutation<T>;
}

impl<T> MutationExt<T> for Mutation<T> {
    // fn checks_ok(self) -> Result<Result<T, CheckError>, MutationError> {
    //     match self {
    //         Ok(r) => Ok(Ok(r)),
    //         Err(MutationError::Check(e)) => Ok(Err(e)),
    //         Err(err) => Err(err),
    //     }
    // }

    fn map_check_err(self, f: impl Fn(CheckMsg) -> CheckMsg) -> Mutation<T> {
        match self {
            Err(MutationError::Check(e)) => Err(MutationError::Check(f(e))),
            other => other,
        }
    }
}

// /// Mutation errors must give String reasons for mutation, which can be used to
// /// specify the error when used for a Check
// #[derive(derive_more::Deref, derive_more::From)]
// pub struct Mutation<T>(MutationResult<T>);

// impl<T> Mutation<T> {
//     pub fn ok(t: T) -> Self {
//         Self(Ok(t))
//     }

//     pub fn result(self) -> MutationResult<T> {
//         self.0
//     }

//     pub fn check_only(self) -> () {
//         todo!()
//     }
// }

// impl<T> From<Result<T, CheckError>> for Mutation<T> {
//     fn from(res: Result<T, CheckError>) -> Self {
//         res.map_err(MutationError::from).into()
//     }
// }

// impl<T> From<Result<T, arbitrary::Error>> for Mutation<T> {
//     fn from(res: Result<T, arbitrary::Error>) -> Self {
//         res.map_err(MutationError::from).into()
//     }
// }

// impl<T> From<Result<T, ContrafactError>> for Mutation<T> {
//     fn from(res: Result<T, ContrafactError>) -> Self {
//         res.map_err(MutationError::from).into()
//     }
// }
