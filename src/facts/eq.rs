use super::*;

/// Specifies an equality constraint
pub fn eq<S, T>(context: S, constant: T) -> EqFact<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq,
{
    EqFact {
        context: context.to_string(),
        constant,
        op: EqOp::Equal,
        _phantom: PhantomData,
    }
}

/// Specifies an equality constraint with no context
pub fn eq_<T>(constant: T) -> EqFact<T>
where
    T: std::fmt::Debug + PartialEq,
{
    eq("eq", constant)
}

/// Specifies an inequality constraint
pub fn ne<S, T>(context: S, constant: T) -> EqFact<T>
where
    S: ToString,
    T: std::fmt::Debug + PartialEq,
{
    EqFact {
        context: context.to_string(),
        constant,
        op: EqOp::NotEqual,
        _phantom: PhantomData,
    }
}

/// Specifies an inequality constraint with no context
pub fn ne_<T>(constant: T) -> EqFact<T>
where
    T: std::fmt::Debug + PartialEq,
{
    ne("ne", constant)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EqFact<T> {
    context: String,
    op: EqOp,
    constant: T,
    _phantom: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EqOp {
    Equal,
    NotEqual,
}

impl<'a, T> Fact<'a, T> for EqFact<T>
where
    T: Bounds<'a> + PartialEq + Clone,
{
    #[tracing::instrument(fields(fact = "eq"), skip(self, g))]
    fn mutate(&mut self, g: &mut Generator<'a>, mut obj: T) -> Mutation<T> {
        let constant = self.constant.clone();
        match self.op {
            EqOp::Equal => {
                if obj != constant {
                    g.fail(format!(
                        "{}: expected {:?} == {:?}",
                        self.context, obj, constant
                    ))?;
                    obj = constant;
                }
            }
            EqOp::NotEqual => loop {
                if obj != constant {
                    break;
                }
                obj = g.arbitrary(format!(
                    "{}: expected {:?} != {:?}",
                    self.context, obj, constant
                ))?;
            },
        }
        Ok(obj)
    }
}
