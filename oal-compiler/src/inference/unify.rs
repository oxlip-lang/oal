use super::disjoin;
use super::tag::{FuncTag, Tag};
use crate::errors::{Error, Kind, Result};
use oal_model::span::Span;

fn occurs(a: &Tag, b: &Tag) -> bool {
    assert!(a.is_variable());
    if a == b {
        true
    } else if let Tag::Func(FuncTag { bindings, range }) = b {
        occurs(a, range) || bindings.iter().any(|binding| occurs(a, binding))
    } else {
        false
    }
}

fn unify(s: &mut disjoin::Set, left: &Tag, right: &Tag) -> Result<()> {
    let left = s.substitute(left);
    let right = s.substitute(right);

    if left == right {
        Ok(())
    } else if let Tag::Var(ref v) = left {
        if occurs(&left, &right) {
            Err(Error::new(Kind::InvalidTypes, "recursive type").with(&(left, right)))
        } else {
            s.extend(v.clone(), right);
            Ok(())
        }
    } else if let Tag::Var(ref v) = right {
        if occurs(&right, &left) {
            Err(Error::new(Kind::InvalidTypes, "recursive type").with(&(right, left)))
        } else {
            s.extend(v.clone(), left);
            Ok(())
        }
    } else if let (
        Tag::Func(FuncTag {
            bindings: left_bindings,
            range: left_range,
        }),
        Tag::Func(FuncTag {
            bindings: right_bindings,
            range: right_range,
        }),
    ) = (&left, &right)
    {
        if left_bindings.len() != right_bindings.len() {
            Err(Error::new(Kind::InvalidTypes, "function arity mismatch")
                .with(&(left_bindings, right_bindings)))
        } else {
            unify(s, left_range, right_range).and_then(|_| {
                left_bindings
                    .iter()
                    .zip(right_bindings.iter())
                    .try_for_each(|(l, r)| unify(s, l, r))
            })
        }
    } else if let (Tag::Property(left_prop), Tag::Property(right_prop)) = (&left, &right) {
        unify(s, left_prop, right_prop)
    } else {
        Err(Error::new(Kind::InvalidTypes, "type mismatch").with(&(left, right)))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct TypeEquation {
    pub left: Tag,
    pub right: Tag,
    pub span: Option<Span>,
}

impl TypeEquation {
    fn unify(&self, s: &mut disjoin::Set) -> Result<()> {
        unify(s, &self.left, &self.right)
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct InferenceSet(Vec<TypeEquation>);

impl InferenceSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, left: Tag, right: Tag, span: Option<Span>) {
        self.0.push(TypeEquation { left, right, span });
    }

    pub fn unify(&self) -> Result<disjoin::Set> {
        let mut s = disjoin::Set::new();
        for eq in self.0.iter() {
            eq.unify(&mut s).map_err(|err| err.at(eq.span.clone()))?;
        }
        Ok(s)
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
