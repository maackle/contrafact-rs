/// The result of a check operation, which contains an error message for every
/// constraint which was not met
// TODO: add ability to abort, so that further checks will not occur
#[derive(derive_more::From, derive_more::IntoIterator)]
#[must_use = "Check should be used with either `.unwrap()` or `.ok()`"]
pub struct Check {
    errors: Vec<String>,
}

impl Check {
    /// Map over each error string
    pub fn map<F>(self, f: F) -> Self
    where
        F: FnMut(String) -> String,
    {
        if let Err(errs) = self.ok() {
            errs.into_iter().map(f).collect()
        } else {
            vec![]
        }
        .into()
    }

    /// Panic if there are any errors, and display those errors
    pub fn unwrap(self) {
        if !self.errors.is_empty() {
            let msg = if self.errors.len() == 1 {
                format!("Check failed: {}", self.errors[0])
            } else {
                format!("Check failed: {:#?}", self.errors)
            };
            panic!(msg);
        }
    }

    /// Convert to a Result: No errors => Ok
    pub fn ok(self) -> std::result::Result<(), Vec<String>> {
        if self.errors.is_empty() {
            std::result::Result::Ok(())
        } else {
            std::result::Result::Err(self.errors)
        }
    }

    /// Create a single-error failure if predicate is false, otherwise pass
    pub fn single<S: ToString>(ok: bool, err: S) -> Self {
        if ok {
            Self::pass()
        } else {
            Self::fail(vec![err.to_string()])
        }
    }

    /// Create an ok result.
    pub fn pass() -> Self {
        Self {
            errors: Vec::with_capacity(0),
        }
    }

    /// Create a failure result.
    pub fn fail(errors: Vec<String>) -> Self {
        Self { errors }
    }
}

type CheckResult = crate::Result<Check>;

impl From<CheckResult> for Check {
    fn from(result: CheckResult) -> Check {
        match result {
            Ok(check) => check,
            Err(err) => vec![err.to_string()].into(),
        }
    }
}

/// Run a check which may produce a Result, mapping any Err into
/// a normal Check error string
#[macro_export]
macro_rules! check_fallible {
    ($blk:block) => {{
        let result: $crate::Result<Check> = (|| $blk)();
        Check::from(result)
    }};
}

#[cfg(test)]
mod tests {
    use crate::Fact;

    use super::*;

    #[test]
    fn test_check_fallible() {
        struct F;
        impl Fact<()> for F {
            fn check(&mut self, _: &()) -> Check {
                check_fallible! {{
                    let x = 1;
                    Ok(if x == 1 {
                        Err(anyhow::Error::msg("oh no"))?
                    } else {
                        Check::pass()
                    })
                }}
            }

            fn mutate(&mut self, _: &mut (), _: &mut arbitrary::Unstructured<'static>) {
                unimplemented!()
            }
        }

        assert_eq!(F.check(&()).ok().unwrap_err(), vec!["oh no"]);
    }
}
