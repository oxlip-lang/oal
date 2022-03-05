use crate::errors::{Error, Result};
use crate::{Env, Scan, Transform};
use oal_syntax::ast::{Expr, FuncTag, Operator, Tag, TypedExpr};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct TagSeq(usize);

impl TagSeq {
    pub fn new() -> Self {
        TagSeq(0)
    }

    pub fn next(&mut self) -> usize {
        let n = self.0;
        self.0 += 1;
        n
    }
}

pub fn tag_type(seq: &mut TagSeq, env: &mut Env, e: &mut TypedExpr) -> Result<()> {
    match &mut e.inner {
        Expr::Prim(_) => {
            e.tag = Some(Tag::Primitive);
            Ok(())
        }
        Expr::Rel(rel) => {
            e.tag = Some(Tag::Relation);
            rel.transform(seq, env, tag_type)
        }
        Expr::Uri(uri) => {
            e.tag = Some(Tag::Uri);
            uri.transform(seq, env, tag_type)
        }
        Expr::Block(block) => {
            e.tag = Some(Tag::Object);
            block.transform(seq, env, tag_type)
        }
        Expr::Op(operation) => {
            e.tag = Some(match operation.op {
                Operator::Join => Tag::Object,
                Operator::Any => Tag::Any,
                Operator::Sum => Tag::Var(seq.next()),
            });
            operation.transform(seq, env, tag_type)
        }
        Expr::Var(var) => match env.lookup(var) {
            None => Err(Error::new("identifier not in scope")),
            Some(val) => {
                e.tag = val.tag.clone();
                Ok(())
            }
        },
        Expr::Lambda(lambda) => {
            e.tag = Some(Tag::Var(seq.next()));
            lambda.transform(seq, env, tag_type)
        }
        Expr::Binding(_) => {
            e.tag = Some(Tag::Var(seq.next()));
            Ok(())
        }
        _ => unreachable!(),
    }
}

#[derive(Debug, Default)]
pub struct Subst(HashMap<usize, Tag>);

impl Subst {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn extend(&mut self, v: usize, t: Tag) {
        self.0.insert(v, t);
    }

    pub fn substitute(&self, t: &Tag) -> Tag {
        let mut tag = t;
        loop {
            if let Tag::Var(v) = tag {
                if let Some(t) = self.0.get(v) {
                    tag = t;
                    continue;
                }
            }
            break;
        }
        tag.clone()
    }
}

pub fn substitute(subst: &mut Subst, _: &mut Env, e: &mut TypedExpr) -> Result<()> {
    e.tag = Some(subst.substitute(e.unwrap_tag()));
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeEquation {
    pub left: Tag,
    pub right: Tag,
}

fn occurs(a: &Tag, b: &Tag) -> bool {
    assert!(a.is_variable());
    // Trivial as we don't have function types yet and therefore Tag is not a recursive type.
    a == b
}

impl TypeEquation {
    pub fn unify(&self, s: &mut Subst) -> bool {
        let left = s.substitute(&self.left);
        let right = s.substitute(&self.right);

        if left == right {
            true
        } else if let Tag::Var(v) = left {
            if occurs(&left, &right) {
                false
            } else {
                s.extend(v, right);
                true
            }
        } else if let Tag::Var(v) = right {
            if occurs(&right, &left) {
                false
            } else {
                s.extend(v, left);
                true
            }
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TypeConstraint(Vec<TypeEquation>);

impl TypeConstraint {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, left: &Tag, right: &Tag) {
        let left = left.clone();
        let right = right.clone();
        self.0.push(TypeEquation { left, right });
    }

    pub fn unify(&self) -> Result<Subst> {
        let mut s = Subst::new();
        for eq in self.0.iter() {
            if !eq.unify(&mut s) {
                return Err(Error::new("cannot unify"));
            }
        }
        Ok(s)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub fn constrain(c: &mut TypeConstraint, env: &mut Env, e: &TypedExpr) -> Result<()> {
    match &e.inner {
        Expr::Prim(_) => c.push(e.unwrap_tag(), &Tag::Primitive),
        Expr::Rel(rel) => {
            rel.scan(c, env, constrain)?;
            c.push(rel.range.unwrap_tag(), &Tag::Object);
            c.push(rel.uri.unwrap_tag(), &Tag::Uri);
            c.push(e.unwrap_tag(), &Tag::Relation);
        }
        Expr::Uri(uri) => {
            uri.scan(c, env, constrain)?;
            for seg in uri.into_iter() {
                c.push(seg.unwrap_tag(), &Tag::Primitive);
            }
            c.push(e.unwrap_tag(), &Tag::Uri);
        }
        Expr::Block(block) => {
            block.scan(c, env, constrain)?;
            c.push(e.unwrap_tag(), &Tag::Object);
        }
        Expr::Op(operation) => {
            operation.scan(c, env, constrain)?;
            let operator = operation.op;
            for op in operation.into_iter() {
                match operator {
                    Operator::Join => c.push(op.unwrap_tag(), &Tag::Object),
                    Operator::Sum => c.push(e.unwrap_tag(), op.unwrap_tag()),
                    _ => {}
                }
            }
            match operator {
                Operator::Join => c.push(e.unwrap_tag(), &Tag::Object),
                Operator::Any => c.push(e.unwrap_tag(), &Tag::Any),
                _ => {}
            }
        }
        Expr::Lambda(lambda) => {
            lambda.scan(c, env, constrain)?;
            let bindings = lambda
                .bindings
                .iter()
                .map(|b| b.unwrap_tag().clone())
                .collect();
            let range = lambda.body.unwrap_tag().clone().into();
            c.push(e.unwrap_tag(), &Tag::Func(FuncTag { bindings, range }));
        }
        Expr::Var(_) => {}
        Expr::Binding(_) => {}
        _ => unreachable!(),
    }
    Ok(())
}
