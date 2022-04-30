use crate::errors::{Error, Kind, Result};
use crate::scope::Env;
use crate::tag::{FuncTag, Tag, Tagged};
use oal_syntax::ast::{AsExpr, Expr, NodeMut, NodeRef, Operator};
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

pub fn tag_type<T>(seq: &mut TagSeq, env: &mut Env<T>, node: NodeMut<T>) -> Result<()>
where
    T: AsExpr + Tagged,
{
    match node {
        NodeMut::Expr(e) => match e.as_ref() {
            Expr::Prim(_) => {
                e.set_tag(Tag::Primitive);
                Ok(())
            }
            Expr::Rel(_) => {
                e.set_tag(Tag::Relation);
                Ok(())
            }
            Expr::Uri(_) => {
                e.set_tag(Tag::Uri);
                Ok(())
            }
            Expr::Object(_) => {
                e.set_tag(Tag::Object);
                Ok(())
            }
            Expr::Array(_) => {
                e.set_tag(Tag::Array);
                Ok(())
            }
            Expr::Op(operation) => {
                let tag = match operation.op {
                    Operator::Join => Tag::Object,
                    Operator::Any => Tag::Any,
                    Operator::Sum => Tag::Var(seq.next()),
                };
                e.set_tag(tag);
                Ok(())
            }
            Expr::Var(var) => match env.lookup(var) {
                None => Err(Error::new(Kind::IdentifierNotInScope, "").with(e)),
                Some(val) => {
                    e.set_tag(val.unwrap_tag());
                    Ok(())
                }
            },
            Expr::Lambda(_) | Expr::Binding(_) => {
                e.set_tag(Tag::Var(seq.next()));
                Ok(())
            }
            Expr::App(application) => match env.lookup(&application.name) {
                None => Err(Error::new(Kind::IdentifierNotInScope, "").with(e)),
                Some(val) => {
                    if let Expr::Lambda(l) = val.as_ref() {
                        e.set_tag(l.body.unwrap_tag());
                        Ok(())
                    } else {
                        Err(Error::new(Kind::IdentifierNotAFunction, "").with(e))
                    }
                }
            },
        },
        _ => Ok(()),
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

    pub fn substitute(&self, tag: &Tag) -> Tag {
        if let Tag::Var(v) = tag {
            if let Some(t) = self.0.get(v) {
                self.substitute(t)
            } else {
                tag.clone()
            }
        } else if let Tag::Func(FuncTag { bindings, range }) = tag {
            let bindings = bindings.iter().map(|b| self.substitute(b)).collect();
            let range = self.substitute(range).into();
            Tag::Func(FuncTag { bindings, range })
        } else {
            tag.clone()
        }
    }
}

pub fn substitute<T: Tagged>(subst: &mut Subst, _env: &mut Env<T>, node: NodeMut<T>) -> Result<()> {
    match node {
        NodeMut::Expr(e) => e.set_tag(subst.substitute(e.tag().unwrap())),
        _ => {}
    }
    Ok(())
}

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

fn unify(s: &mut Subst, left: &Tag, right: &Tag) -> bool {
    let left = s.substitute(left);
    let right = s.substitute(right);

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
    } else if let (
        Tag::Func(FuncTag {
            bindings: left_bindings,
            range: left_range,
        }),
        Tag::Func(FuncTag {
            bindings: right_bindings,
            range: right_range,
        }),
    ) = (left, right)
    {
        if left_bindings.len() != right_bindings.len() {
            false
        } else {
            unify(s, &left_range, &right_range)
                && left_bindings
                    .iter()
                    .zip(right_bindings.iter())
                    .all(|(l, r)| unify(s, l, r))
        }
    } else {
        false
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeEquation {
    pub left: Tag,
    pub right: Tag,
}

impl TypeEquation {
    pub fn unify(&self, s: &mut Subst) -> bool {
        unify(s, &self.left, &self.right)
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TypeConstraint(Vec<TypeEquation>);

impl TypeConstraint {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, left: Tag, right: Tag) {
        self.0.push(TypeEquation { left, right });
    }

    pub fn unify(&self) -> Result<Subst> {
        let mut s = Subst::new();
        for eq in self.0.iter() {
            if !eq.unify(&mut s) {
                return Err(Error::new(Kind::CannotUnify, "").with(eq));
            }
        }
        Ok(s)
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub fn constrain<T>(c: &mut TypeConstraint, env: &mut Env<T>, node: NodeRef<T>) -> Result<()>
where
    T: AsExpr + Tagged,
{
    match node {
        NodeRef::Expr(e) => match e.as_ref() {
            Expr::Prim(_) => {
                c.push(e.unwrap_tag(), Tag::Primitive);
                Ok(())
            }
            Expr::Rel(rel) => {
                for xfer in rel.xfers.values() {
                    if let Some(x) = xfer.as_ref() {
                        c.push(x.range.unwrap_tag(), Tag::Object);
                        if let Some(domain) = &x.domain {
                            c.push(domain.unwrap_tag(), Tag::Object);
                        }
                    }
                }
                c.push(rel.uri.unwrap_tag(), Tag::Uri);
                c.push(e.unwrap_tag(), Tag::Relation);
                Ok(())
            }
            Expr::Uri(uri) => {
                for seg in uri.into_iter() {
                    c.push(seg.unwrap_tag(), Tag::Primitive);
                }
                c.push(e.unwrap_tag(), Tag::Uri);
                Ok(())
            }
            Expr::Object(_) => {
                c.push(e.unwrap_tag(), Tag::Object);
                Ok(())
            }
            Expr::Array(_) => {
                c.push(e.unwrap_tag(), Tag::Array);
                Ok(())
            }
            Expr::Op(operation) => {
                let operator = operation.op;
                for op in operation.into_iter() {
                    match operator {
                        Operator::Join => c.push(op.unwrap_tag(), Tag::Object),
                        Operator::Sum => c.push(e.unwrap_tag(), op.unwrap_tag()),
                        _ => {}
                    }
                }
                match operator {
                    Operator::Join => c.push(e.unwrap_tag(), Tag::Object),
                    Operator::Any => c.push(e.unwrap_tag(), Tag::Any),
                    _ => {}
                }
                Ok(())
            }
            Expr::Lambda(lambda) => {
                let bindings = lambda
                    .bindings
                    .iter()
                    .map(|b| b.unwrap_tag().clone())
                    .collect();
                let range = lambda.body.unwrap_tag().clone().into();
                c.push(e.unwrap_tag(), Tag::Func(FuncTag { bindings, range }));
                Ok(())
            }
            Expr::App(application) => match env.lookup(&application.name) {
                None => Err(Error::new(Kind::IdentifierNotInScope, "").with(e)),
                Some(val) => {
                    let bindings = application
                        .args
                        .iter()
                        .map(|a| a.unwrap_tag().clone())
                        .collect();
                    let range = e.unwrap_tag().clone().into();
                    c.push(val.unwrap_tag(), Tag::Func(FuncTag { bindings, range }));
                    Ok(())
                }
            },
            Expr::Var(_) => Ok(()),
            Expr::Binding(_) => Ok(()),
        },
        _ => Ok(()),
    }
}
