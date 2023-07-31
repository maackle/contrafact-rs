use crate::*;

/// The result of a check operation, which contains a failure message for every
/// constraint which was not met.
///
/// There are two levels of "error" here: the failures due to data which does not
/// meet the constraints, and also internal errors due to a poorly written Fact.
//
// TODO: add ability to abort, so that further checks will not occur
#[derive(Debug, Clone, PartialEq, Eq, derive_more::From)]
#[must_use = "Check should be used with either `.unwrap()` or `.result()`"]
pub enum Check {
    /// The check ran successfully, and reported these failures.
    /// An empty list of failures means the data is valid per this check.
    Failures(Vec<Failure>),

    /// There was a problem actually running the check: there is a bug in a Fact
    /// or Generator.
    //
    // TODO: convert to ContrafactError, if this PR is merged:
    // https://github.com/rust-fuzz/arbitrary/pull/153
    Error(String),
}

impl Check {
    /// Map over each failure string.
    /// Useful for combinators which add additional context to errors produced
    /// by inner facts.
    pub fn map<F>(self, f: F) -> Self
    where
        F: FnMut(Failure) -> Failure,
    {
        match self {
            Self::Failures(failures) => Self::Failures(failures.into_iter().map(f).collect()),
            e => e,
        }
    }

    /// Panic if there are any errors, and display those errors.
    pub fn unwrap(self) {
        match self {
            Self::Failures(failures) => {
                if !failures.is_empty() {
                    if failures.len() == 1 {
                        panic!("Check failed: {}", failures[0])
                    } else {
                        panic!("Check failed: {:#?}", failures)
                    };
                }
            }
            Self::Error(err) => panic!("Internal contrafact error. Check your Facts! {:?}", err),
        }
    }

    /// There are no errors.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Failures(failures) if failures.is_empty())
    }

    /// There is at least one error.
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Get errors if they exist
    pub fn failures(&self) -> Result<&[Failure], ContrafactError> {
        match self {
            Self::Failures(failures) => Ok(failures.as_ref()),
            Self::Error(err) => Err(err.clone().into()),
        }
    }

    /// Convert to a Result: No failures => `Ok` all the way
    /// The result is wrapped in another Result, in case the overall check failed for an internal reason
    ///
    /// ```
    /// use contrafact::*;
    /// assert_eq!(Check::pass().result(), Ok(Ok(())));
    /// assert_eq!(Check::fail("message").result(), Ok(Err(vec!["message".to_string()])));
    /// ```
    pub fn result(self) -> ContrafactResult<std::result::Result<(), Vec<Failure>>> {
        match self {
            Self::Failures(failures) => {
                if failures.is_empty() {
                    Ok(Ok(()))
                } else {
                    Ok(Err(failures))
                }
            }
            Self::Error(err) => Err(err.into()),
        }
    }

    /// If Failures, return all failures joined together in a single string
    pub fn result_joined(self) -> ContrafactResult<std::result::Result<(), String>> {
        self.result().map(|r| r.map_err(|es| es.join(";")))
    }

    /// Create a single-error failure if predicate is false, otherwise pass
    ///
    /// ```
    /// use contrafact::*;
    /// assert_eq!(Check::check(true, "message"), Check::pass());
    /// assert_eq!(Check::check(false, "message"), Check::fail("message"));
    /// ```
    pub fn check<S: ToString>(ok: bool, err: S) -> Self {
        if ok {
            Self::pass()
        } else {
            Self::fail(err)
        }
    }

    /// Create a single-error failure from a Result
    ///
    /// ```
    /// use contrafact::*;
    /// assert_eq!(Check::from_mutation(Ok(42)), Check::pass());
    /// assert_eq!(Check::from_mutation::<()>(Err(MutationError::Check("message".to_string()))), Check::fail("message"));
    /// ```
    pub fn from_mutation<T>(res: Mutation<T>) -> Self {
        match res {
            Ok(_) => Self::pass(),
            Err(MutationError::Check(err)) => Self::fail(err),
            Err(MutationError::Arbitrary(err)) => Self::Error(err.to_string()),
            Err(MutationError::Internal(err)) => Self::Error(format!("{:?}", err)),
            Err(MutationError::User(err)) => Self::Error(format!("{:?}", err)),
        }
    }

    /// Create a check where failures are drawn from Ok, and internal errors from Err of the input Result
    pub fn from_result(res: Result<Vec<Failure>, ContrafactError>) -> Self {
        res.map(Self::Failures)
            .unwrap_or_else(|e| Self::Error(format!("{:?}", e)))
    }

    /// Create an ok result.
    ///
    /// ```
    /// use contrafact::*;
    /// assert_eq!(Check::pass(), vec![].into())
    /// ```
    pub fn pass() -> Self {
        Self::Failures(Vec::with_capacity(0))
    }

    /// Create a failure result with a single error.
    ///
    /// ```
    /// use contrafact::*;
    /// assert_eq!(Check::fail("message"), vec!["message".to_string()].into())
    /// ```
    pub fn fail<S: ToString>(error: S) -> Self {
        Self::Failures(vec![error.to_string()])
    }
}
