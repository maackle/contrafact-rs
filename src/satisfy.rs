use crate::*;

/// Check that all of the constraints of all Facts are satisfied for this sequence.
/// Each Fact will run [`Fact::advance`] after each item checked, allowing stateful
/// facts to change as the sequence advances.
#[tracing::instrument(skip(fact))]
pub fn check_seq<'a, T, F>(seq: &[T], mut fact: F) -> Check
where
    F: Fact<'a, T>,
    T: Bounds<'a>,
{
    let mut reasons: Vec<String> = Vec::new();
    for (i, obj) in seq.iter().enumerate() {
        match fact.check(obj).failures() {
            Ok(failures) => {
                reasons.extend(
                    failures
                        .into_iter()
                        .map(|reason| format!("item {}: {}", i, reason))
                        .collect::<Vec<_>>(),
                );
            }
            Err(err) => return Check::Error(format!("{:?}", err)),
        }
        fact.advance(obj);
    }
    reasons.into()
}

/// Build a sequence from scratch such that all Facts are satisfied.
/// Each Fact will run [`Fact::advance`] after each item built, allowing stateful
/// facts to change as the sequence advances.
#[tracing::instrument(skip(g, fact))]
pub fn build_seq_fallible<'a, T, F>(
    g: &mut Generator<'a>,
    num: usize,
    mut fact: F,
) -> ContrafactResult<Vec<T>>
where
    T: Bounds<'a>,
    F: Fact<'a, T>,
{
    let mut seq = Vec::new();
    for _i in 0..num {
        tracing::trace!("i: {}", _i);
        let obj = fact.build_fallible(g)?;
        fact.advance(&obj);
        seq.push(obj);
    }
    Ok(seq)
}

/// Build a sequence from scratch such that all Facts are satisfied.
/// Each Fact will run [`Fact::advance`] after each item built, allowing stateful
/// facts to change as the sequence advances.
///
/// ## Panics
///
/// Panics if an error is encountered during any build step. To handle these errors,
/// use [`build_seq_fallible`] instead.

#[tracing::instrument(skip(g, fact))]
pub fn build_seq<'a, T, F>(g: &mut Generator<'a>, num: usize, fact: F) -> Vec<T>
where
    T: Bounds<'a>,
    F: Fact<'a, T>,
{
    build_seq_fallible(g, num, fact).unwrap()
}

/// Build an infinite iterator of items such that all Facts are satisfied.
/// Each Fact will run [`Fact::advance`] after each item built, allowing stateful
/// facts to change as the sequence advances.
///
/// ## Panics
///
/// Panics if an error is encountered at any iteration.
#[tracing::instrument(skip(g, fact))]
pub fn build_iter<'a, T, F>(mut g: Generator<'a>, mut fact: F) -> impl Iterator<Item = T> + 'a
where
    T: Bounds<'a>,
    F: 'a + Fact<'a, T>,
{
    // (0..).scan((g, fact), |(g, fact), _| {
    //     let obj = fact.build_fallible(g).unwrap();
    //     fact.advance(&obj);
    //     Some(obj)
    // })

    std::iter::repeat_with(move || {
        let obj = fact.build_fallible(&mut g).unwrap();
        fact.advance(&obj);
        obj
    })
}

// /// Build an infinite iterator of items such that all Facts are satisfied.
// /// Each Fact will run [`Fact::advance`] after each item built, allowing stateful
// /// facts to change as the sequence advances.
// ///
// /// ## Panics
// ///
// /// Panics if an error is encountered at any iteration.
// #[tracing::instrument(skip(g, fact))]
// pub fn build_iter<'a, 'b: 'a, T, F>(
//     g: &'b mut Generator<'a>,
//     mut fact: F,
// ) -> impl Iterator<Item = T> + 'b
// where
//     T: 'b + Bounds<'a>,
//     F: 'b + Fact<'a, T>,
// {
//     let repeater = move || {
//         let obj = fact.build_fallible(g).unwrap();
//         fact.advance(&obj);
//         obj
//     };
//     std::iter::repeat_with(repeater)
// }

/// Convenience macro for creating a collection of [`Fact`](crate::Fact)s
/// of different types.
/// Each Fact will be boxed and added to a Vec as a trait object, with their
/// types erased.
/// The resulting value also implements `Fact`.
///
/// ```
/// use contrafact::*;
///
/// let eq1 = eq_(1);
/// let not2 = not_(eq_(2));
/// let fact: FactsRef<'static, u32> = facts![eq1, not2];
/// assert!(fact.check(&1).is_ok());
/// ```
#[macro_export]
macro_rules! facts {
    ( $( $fact:expr ),+ $(,)?) => {{
        let mut fs: $crate::FactsRef<_> = Vec::new();
        $(
            fs.push(Box::new($fact));
        )+
        fs
    }};
}
