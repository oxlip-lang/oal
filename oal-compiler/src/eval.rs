use crate::errors::{Error, Result};
use crate::inference::{constrain, substitute, tag_type, TagSeq, TypeConstraint};
use crate::reduce;
use crate::scan::Scan;
use crate::scope::Env;
use crate::transform::Transform;
use oal_syntax::ast;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum UriSegment {
    Literal(ast::Literal),
    Variable(Prop),
}

impl TryFrom<&ast::UriSegment> for UriSegment {
    type Error = Error;

    fn try_from(s: &ast::UriSegment) -> Result<Self> {
        match s {
            ast::UriSegment::Literal(l) => Ok(UriSegment::Literal(l.clone())),
            ast::UriSegment::Variable(p) => p.try_into().map(|p| UriSegment::Variable(p)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Uri {
    pub spec: Vec<UriSegment>,
}

impl Uri {
    pub fn pattern(&self) -> String {
        self.spec
            .iter()
            .map(|s| match s {
                UriSegment::Literal(l) => format!("/{}", l),
                UriSegment::Variable(t) => format!("/{{{}}}", t.name),
            })
            .collect()
    }
}

impl TryFrom<&ast::Uri> for Uri {
    type Error = Error;

    fn try_from(uri: &ast::Uri) -> Result<Self> {
        let spec: Result<Vec<UriSegment>> = uri.spec.iter().map(|s| s.try_into()).collect();
        spec.map(|spec| Uri { spec })
    }
}

impl TryFrom<&ast::Expr> for Uri {
    type Error = Error;

    fn try_from(e: &ast::Expr) -> Result<Self> {
        match e {
            ast::Expr::Uri(uri) => uri.try_into(),
            _ => Err(Error::new("expected uri expression").with_expr(e)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Array {
    pub item: Box<Schema>,
}

impl TryFrom<&ast::Array> for Array {
    type Error = Error;

    fn try_from(a: &ast::Array) -> Result<Self> {
        let item = &a.item.inner;
        item.try_into().map(|item| Array {
            item: Box::new(item),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariadicOp {
    pub op: ast::Operator,
    pub schemas: Vec<Schema>,
}

impl TryFrom<&ast::VariadicOp> for VariadicOp {
    type Error = Error;

    fn try_from(op: &ast::VariadicOp) -> Result<Self> {
        let schemas: Result<Vec<_>> = op.exprs.iter().map(|e| (&e.inner).try_into()).collect();
        schemas.map(|schemas| VariadicOp { op: op.op, schemas })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Schema {
    Prim(ast::Prim),
    Rel(Relation),
    Uri(Uri),
    Array(Array),
    Object(Object),
    Op(VariadicOp),
}

impl TryFrom<&ast::Expr> for Schema {
    type Error = Error;

    fn try_from(e: &ast::Expr) -> Result<Self> {
        match e {
            ast::Expr::Prim(prim) => Ok(Schema::Prim(*prim)),
            ast::Expr::Rel(rel) => rel.try_into().map(|r| Schema::Rel(r)),
            ast::Expr::Uri(uri) => uri.try_into().map(|u| Schema::Uri(u)),
            ast::Expr::Array(arr) => arr.try_into().map(|a| Schema::Array(a)),
            ast::Expr::Block(blk) => blk.try_into().map(|o| Schema::Object(o)),
            ast::Expr::Op(op) => op.try_into().map(|o| Schema::Op(o)),
            ast::Expr::Var(_) => Err(Error::new("unexpected variable expression").with_expr(e)),
            ast::Expr::Lambda(_) => Err(Error::new("unexpected lambda expression").with_expr(e)),
            ast::Expr::App(_) => Err(Error::new("unexpected application expression").with_expr(e)),
            ast::Expr::Binding(_) => Err(Error::new("unexpected binding expression").with_expr(e)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prop {
    pub name: ast::Ident,
    pub schema: Schema,
}

impl TryFrom<&ast::Prop> for Prop {
    type Error = Error;

    fn try_from(p: &ast::Prop) -> Result<Self> {
        let val = &p.val.inner;
        val.try_into().map(|s| Prop {
            name: p.key.clone(),
            schema: s,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Object {
    pub props: Vec<Prop>,
}

impl TryFrom<&ast::Block> for Object {
    type Error = Error;

    fn try_from(b: &ast::Block) -> Result<Self> {
        let props: Result<Vec<_>> = b.props.iter().map(|p| p.try_into()).collect();
        props.map(|props| Object { props })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Operation {
    pub domain: Option<Schema>,
    pub range: Schema,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Relation {
    pub uri: Uri,
    pub ops: HashMap<ast::Method, Operation>,
}

impl TryFrom<&ast::Rel> for Relation {
    type Error = Error;

    fn try_from(r: &ast::Rel) -> Result<Self> {
        let uri: Uri = (&r.uri.inner).try_into()?;
        let range: Schema = (&r.range.inner).try_into()?;
        let domain: Option<Schema> = match &r.domain {
            Some(d) => (&d.inner).try_into().map(|d| Some(d)),
            None => Ok(None),
        }?;
        let op = Operation { range, domain };
        let ops = r.methods.iter().map(|m| (*m, op.clone())).collect();
        Ok(Relation { uri, ops })
    }
}

impl TryFrom<&ast::Expr> for Relation {
    type Error = Error;

    fn try_from(e: &ast::Expr) -> Result<Self> {
        match e {
            ast::Expr::Rel(rel) => rel.try_into(),
            _ => Err(Error::new("expected relation expression").with_expr(e)),
        }
    }
}

pub type PathPattern = String;
pub type Relations = HashMap<PathPattern, Relation>;

#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    pub rels: Relations,
}

impl TryFrom<&ast::Doc> for Spec {
    type Error = Error;

    fn try_from(doc: &ast::Doc) -> Result<Self> {
        let mut rels: Relations = HashMap::new();

        doc.stmts.iter().try_for_each(|stmt| match stmt {
            ast::Stmt::Res(res) => Relation::try_from(&res.rel.inner).and_then(|rel| {
                match rels.entry(rel.uri.pattern()) {
                    Entry::Vacant(v) => {
                        v.insert(rel);
                        Ok(())
                    }
                    Entry::Occupied(_) => Err(Error::new("relation conflict")),
                }
            }),
            _ => Ok(()),
        })?;

        Ok(Spec { rels })
    }
}

pub fn evaluate(mut doc: ast::Doc) -> Result<Spec> {
    doc.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)?;

    let constraint = &mut TypeConstraint::new();

    doc.scan(constraint, &mut Env::new(), constrain)?;

    let subst = &mut constraint.unify()?;

    doc.transform(subst, &mut Env::new(), substitute)?;

    doc.transform(&mut (), &mut Env::new(), reduce)?;

    Spec::try_from(&doc)
}
