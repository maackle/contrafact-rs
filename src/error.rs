/// A failure is the reason why some data does not conform to a given Fact
pub type Failure = String;

// ///
// #[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
// pub enum CheckError {
//     Check(Failure),
//     Internal(ContrafactError),
// }

/// Errors caused by bugs in Facts, Generators, or contrafact itself
#[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
pub enum ContrafactError {
    // TODO: uncomment if this PR is merged:
    // https://github.com/rust-fuzz/arbitrary/pull/153
    // UnexpectedError(arbitrary::Error),
    /// The only type of error we know about right now
    Other(String),
}

/// Alias
pub type ContrafactResult<T> = Result<T, ContrafactError>;

/// Errors which can occur during a `mutate()` call
#[derive(Clone, Debug, derive_more::From)]
pub enum MutationError {
    /// When running check, this is a failure which was generated instead of mutating the data
    Check(Failure),
    /// arbitrary failed to produce new data, which means we can't go on
    #[from]
    Arbitrary(arbitrary::Error),

    /// Contrafact experienced a problem
    #[from]
    Internal(ContrafactError),

    /// There was some other bug in the Fact implementation
    User(String),
}

impl PartialEq for MutationError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Check(s), Self::Check(o)) => s == o,
            (Self::Arbitrary(s), Self::Arbitrary(o)) => s.to_string() == o.to_string(),
            (Self::Internal(s), Self::Internal(o)) => s == o,
            (Self::User(s), Self::User(o)) => s == o,
            _ => false,
        }
    }
}

/// Alias
pub type Mutation<T> = Result<T, MutationError>;

/// Adds a helpful method to MutationResults
pub trait MutationExt<T> {
    /// Map over only the Failures, leaving other error kinds untouched
    fn map_check_err(self, f: impl Fn(Failure) -> Failure) -> Mutation<T>;
}

impl<T> MutationExt<T> for Mutation<T> {
    fn map_check_err(self, f: impl Fn(Failure) -> Failure) -> Mutation<T> {
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
