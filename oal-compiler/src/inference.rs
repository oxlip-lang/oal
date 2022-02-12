use crate::errors::{Error, Result};
use crate::Env;
use oal_syntax::ast::{
    Composite, Decl, Doc, Expr, Operator, Res, Stmt, Tag, TypedExpr, UriSegment,
};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Subst(HashMap<usize, Tag>);

impl Subst {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn extend(&mut self, v: usize, t: Tag) {
        self.0.insert(v, t);
    }

    pub fn substitute(&self, t: Tag) -> Tag {
        let mut tag = &t;
        loop {
            if let Tag::Var(v) = tag {
                if let Some(t) = self.0.get(v) {
                    tag = t;
                    continue;
                }
            }
            break;
        }
        *tag
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TypeEquation {
    pub left: Tag,
    pub right: Tag,
}

fn occurs(a: Tag, b: Tag) -> bool {
    assert!(a.is_variable());
    // Trivial as we don't have function types yet and therefore Tag is not a recursive type.
    a == b
}

impl TypeEquation {
    pub fn unify(&self, s: &mut Subst) -> bool {
        let left = s.substitute(self.left);
        let right = s.substitute(self.right);

        if left == right {
            true
        } else if let Tag::Var(v) = left {
            if occurs(left, right) {
                false
            } else {
                s.extend(v, right);
                true
            }
        } else if let Tag::Var(v) = right {
            if occurs(right, left) {
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

    pub fn push(&mut self, left: Tag, right: Tag) {
        self.0.push(TypeEquation { left, right });
    }

    pub fn unify(&self) -> Option<Subst> {
        let mut s = Subst::new();
        for eq in self.0.iter() {
            if !eq.unify(&mut s) {
                return None;
            }
        }
        Some(s)
    }
}

pub trait TypeConstrained {
    fn constrain(&self, c: &mut TypeConstraint);
}

impl TypeConstrained for TypedExpr {
    fn constrain(&self, c: &mut TypeConstraint) {
        match &self.expr {
            Expr::Prim(_) => c.push(self.tag.unwrap(), Tag::Primitive),
            Expr::Rel(rel) => {
                rel.range.constrain(c);
                rel.uri.constrain(c);
                c.push(rel.range.tag.unwrap(), Tag::Object);
                c.push(rel.uri.tag.unwrap(), Tag::Uri);
                c.push(self.tag.unwrap(), Tag::Relation);
            }
            Expr::Uri(uri) => {
                for s in uri.spec.iter() {
                    match s {
                        UriSegment::Literal(_) => {}
                        UriSegment::Template(tpl) => {
                            tpl.val.constrain(c);
                            c.push(tpl.val.tag.unwrap(), Tag::Primitive);
                        }
                    }
                }
                c.push(self.tag.unwrap(), Tag::Uri);
            }
            Expr::Block(block) => {
                for prop in block.props.iter() {
                    prop.val.constrain(c);
                }
                c.push(self.tag.unwrap(), Tag::Object);
            }
            Expr::Op(operation) => {
                for expr in operation.exprs.iter() {
                    expr.constrain(c);
                    match operation.op {
                        Operator::Join => c.push(expr.tag.unwrap(), Tag::Object),
                        Operator::Sum => c.push(self.tag.unwrap(), expr.tag.unwrap()),
                        _ => {}
                    }
                }
                match operation.op {
                    Operator::Join => c.push(self.tag.unwrap(), Tag::Object),
                    Operator::Any => c.push(self.tag.unwrap(), Tag::Any),
                    _ => {}
                }
            }
            Expr::Var(_) => {}
        }
    }
}

impl TypeConstrained for Decl {
    fn constrain(&self, c: &mut TypeConstraint) {
        self.body.constrain(c);
    }
}

impl TypeConstrained for Res {
    fn constrain(&self, c: &mut TypeConstraint) {
        self.rel.constrain(c)
    }
}

impl TypeConstrained for Stmt {
    fn constrain(&self, c: &mut TypeConstraint) {
        match self {
            Stmt::Decl(d) => d.constrain(c),
            Stmt::Res(r) => r.constrain(c),
        }
    }
}

impl TypeConstrained for Doc {
    fn constrain(&self, c: &mut TypeConstraint) {
        for s in self.stmts.iter() {
            s.constrain(c);
        }
    }
}

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

pub trait TypeTagged {
    fn tag_type(&mut self, n: &mut TagSeq, e: &mut Env) -> Result<()>;
}

impl TypeTagged for TypedExpr {
    fn tag_type(&mut self, n: &mut TagSeq, e: &mut Env) -> Result<()> {
        match &mut self.expr {
            Expr::Prim(_) => {
                self.tag = Some(Tag::Primitive);
                Ok(())
            }
            Expr::Rel(rel) => {
                self.tag = Some(Tag::Relation);
                rel.foreach(|c| c.tag_type(n, e))
            }
            Expr::Uri(uri) => {
                self.tag = Some(Tag::Uri);
                uri.foreach(|c| c.tag_type(n, e))
            }
            Expr::Block(block) => {
                self.tag = Some(Tag::Object);
                block.foreach(|c| c.tag_type(n, e))
            }
            Expr::Op(operation) => {
                self.tag = Some(match operation.op {
                    Operator::Join => Tag::Object,
                    Operator::Any => Tag::Any,
                    Operator::Sum => Tag::Var(n.next()),
                });
                operation.foreach(|c| c.tag_type(n, e))
            }
            Expr::Var(var) => match e.lookup(var) {
                None => Err(Error::new("identifier not in scope")),
                Some(expr) => {
                    self.tag = expr.tag;
                    Ok(())
                }
            },
        }
    }
}

impl TypeTagged for Decl {
    fn tag_type(&mut self, n: &mut TagSeq, e: &mut Env) -> Result<()> {
        self.body.tag_type(n, e).and_then(|_| {
            e.declare(&self.var, &self.body);
            Ok(())
        })
    }
}

impl TypeTagged for Res {
    fn tag_type(&mut self, n: &mut TagSeq, e: &mut Env) -> Result<()> {
        self.rel.tag_type(n, e)
    }
}

impl TypeTagged for Stmt {
    fn tag_type(&mut self, n: &mut TagSeq, e: &mut Env) -> Result<()> {
        match self {
            Stmt::Decl(d) => d.tag_type(n, e),
            Stmt::Res(r) => r.tag_type(n, e),
        }
    }
}

impl TypeTagged for Doc {
    fn tag_type(&mut self, n: &mut TagSeq, e: &mut Env) -> Result<()> {
        e.open();
        let r = self.foreach(|s| s.tag_type(n, e));
        e.close();
        r
    }
}
