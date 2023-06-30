use super::*;

/// Specifies an equality constraint between two items in a tuple
pub fn same<T>() -> SameFact<T>
where
    T: std::fmt::Debug + PartialEq,
{
    SameFact {
        op: EqOp::Equal,
        _phantom: PhantomData,
    }
}

/// Specifies an inequality constraint between two items in a tuple
pub fn different<T>() -> SameFact<T>
where
    T: std::fmt::Debug + PartialEq,
{
    SameFact {
        op: EqOp::NotEqual,
        _phantom: PhantomData,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SameFact<T> {
    op: EqOp,
    _phantom: PhantomData<T>,
}

impl<'a, T> Factual<'a, (T, T)> for SameFact<T>
where
    T: Bounds<'a> + PartialEq + Clone,
{
    #[tracing::instrument(fields(fact = "same"), skip(self, g))]
    fn mutate(&mut self, g: &mut Generator<'a>, mut obj: (T, T)) -> Mutation<(T, T)> {
        match self.op {
            EqOp::Equal => {
                if obj.0 != obj.1 {
                    g.fail(format!("must be same: expected {:?} == {:?}", obj.0, obj.1))?;
                    obj.0 = obj.1.clone();
                }
            }
            EqOp::NotEqual => loop {
                if obj.0 != obj.1 {
                    break;
                }
                obj.0 = g.arbitrary(format!(
                    "must be different: expected {:?} != {:?}",
                    obj.0, obj.1
                ))?;
            },
        }
        Ok(obj)
    }
}
