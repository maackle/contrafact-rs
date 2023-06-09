pub type CheckError = String;

/// The result of a check operation, which contains an error message for every
/// constraint which was not met.
//
// TODO: add ability to abort, so that further checks will not occur
#[derive(Debug, Clone, PartialEq, Eq, Hash, derive_more::From, derive_more::IntoIterator)]
#[must_use = "Check should be used with either `.unwrap()` or `.result()`"]
pub struct Check {
    errors: Vec<CheckError>,
}

impl Check {
    /// Map over each error string.
    /// Useful for combinators which add additional context to errors produced
    /// by inner facts.
    pub fn map<F>(self, f: F) -> Self
    where
        F: FnMut(CheckError) -> CheckError,
    {
        if let Err(errs) = self.result() {
            errs.into_iter().map(f).collect()
        } else {
            vec![]
        }
        .into()
    }

    /// Panic if there are any errors, and display those errors.
    pub fn unwrap(self) {
        if !self.errors.is_empty() {
            if self.errors.len() == 1 {
                panic!("Check failed: {}", self.errors[0])
            } else {
                panic!("Check failed: {:#?}", self.errors)
            };
        }
    }

    /// There are no errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// There is at least one error.
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Get errors if they exist
    pub fn errors(&self) -> &[CheckError] {
        self.errors.as_ref()
    }

    /// Convert to a Result: No errors => Ok
    ///
    /// ```
    /// use contrafact::*;
    /// assert_eq!(Check::pass().result(), Ok(()));
    /// assert_eq!(Check::fail("message").result(), Err(vec!["message".to_string()]));
    /// ```
    pub fn result(self) -> std::result::Result<(), Vec<CheckError>> {
        if self.is_ok() {
            std::result::Result::Ok(())
        } else {
            std::result::Result::Err(self.errors)
        }
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
    /// assert_eq!(Check::from_result(Ok(42)), Check::pass());
    /// assert_eq!(Check::from_result::<()>(Err("message".to_string())), Check::fail("message"));
    /// ```
    pub fn from_result<T>(res: Result<T, CheckError>) -> Self {
        if let Err(err) = res {
            Self::fail(err)
        } else {
            Self::pass()
        }
    }

    /// Create an ok result.
    ///
    /// ```
    /// use contrafact::*;
    /// assert_eq!(Check::pass(), vec![].into())
    /// ```
    pub fn pass() -> Self {
        Self {
            errors: Vec::with_capacity(0),
        }
    }

    /// Create a failure result with a single error.
    ///
    /// ```
    /// use contrafact::*;
    /// assert_eq!(Check::fail("message"), vec!["message".to_string()].into())
    /// ```
    pub fn fail<S: ToString>(error: S) -> Self {
        Self {
            errors: vec![error.to_string()],
        }
    }
}
