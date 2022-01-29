use crate::Env;
use oal_syntax::ast::{Decl, Doc, Expr, Prim, Res, Stmt, Tag, TypedExpr, UriSegment};

pub struct TypeEquation {
    pub left: Tag,
    pub right: Tag,
}

pub trait TypedNode {
    fn equations(&self, eqs: &mut Vec<TypeEquation>);
}

impl TypedNode for Expr {
    fn equations(&self, _eqs: &mut Vec<TypeEquation>) {
        match self {
            Expr::Prim(_) => {}
            Expr::Rel(_) => {}
            Expr::Uri(_) => {}
            Expr::Join(_) => {}
            Expr::Block(_) => {}
            Expr::Sum(_) => {}
            Expr::Var(_) => {}
        }
    }
}

impl TypedNode for Decl {
    fn equations(&self, eqs: &mut Vec<TypeEquation>) {
        self.body.expr.equations(eqs);
        eqs.push(TypeEquation {
            left: todo!(),
            right: todo!(),
        })
    }
}

impl TypedNode for Res {
    fn equations(&self, eqs: &mut Vec<TypeEquation>) {
        self.rel.expr.equations(eqs)
    }
}

impl TypedNode for Stmt {
    fn equations(&self, eqs: &mut Vec<TypeEquation>) {
        match self {
            Stmt::Decl(d) => d.equations(eqs),
            Stmt::Res(r) => r.equations(eqs),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TagSeq(usize);

impl TagSeq {
    pub fn new() -> TagSeq {
        TagSeq(0)
    }
    pub fn next(&mut self) -> usize {
        let n = self.0;
        self.0 += 1;
        n
    }
}

pub trait Tagged {
    fn tag_type(&mut self, n: &mut TagSeq, e: &mut Env);
}

impl Tagged for TypedExpr {
    fn tag_type(&mut self, n: &mut TagSeq, e: &mut Env) {
        match &mut self.expr {
            Expr::Prim(prim) => {
                let tag = match prim {
                    Prim::Num => Tag::Number,
                    Prim::Str => Tag::String,
                    Prim::Bool => Tag::Boolean,
                };
                self.tag = Some(tag);
            }
            Expr::Rel(rel) => {
                self.tag = Some(Tag::Relation);
                rel.range.tag_type(n, e);
                rel.uri.tag_type(n, e);
            }
            Expr::Uri(uri) => {
                self.tag = Some(Tag::Uri);
                for spec in uri.spec.iter_mut() {
                    match spec {
                        UriSegment::Literal(_) => {}
                        UriSegment::Template(t) => t.val.tag_type(n, e),
                    }
                }
            }
            Expr::Join(join) => {
                self.tag = Some(Tag::Object);
                for expr in join.exprs.iter_mut() {
                    expr.tag_type(n, e);
                }
            }
            Expr::Block(block) => {
                self.tag = Some(Tag::Object);
                for prop in block.props.iter_mut() {
                    prop.val.tag_type(n, e);
                }
            }
            Expr::Sum(sum) => {
                self.tag = Some(Tag::Var(n.next()));
                for expr in sum.exprs.iter_mut() {
                    expr.tag_type(n, e);
                }
            }
            Expr::Var(var) => {
                // TODO: return error instead
                let expr = e.lookup(var).expect("variable not in scope");
                self.tag = expr.tag;
            }
        };
    }
}

impl Tagged for Decl {
    fn tag_type(&mut self, n: &mut TagSeq, e: &mut Env) {
        self.body.tag_type(n, e);
        e.declare(&self.var, &self.body);
    }
}

impl Tagged for Res {
    fn tag_type(&mut self, n: &mut TagSeq, e: &mut Env) {
        self.rel.tag_type(n, e);
    }
}

impl Tagged for Doc {
    fn tag_type(&mut self, n: &mut TagSeq, e: &mut Env) {
        e.open();
        for s in self.stmts.iter_mut() {
            match s {
                Stmt::Decl(d) => d.tag_type(n, e),
                Stmt::Res(r) => r.tag_type(n, e),
            }
        }
        e.close();
    }
}
