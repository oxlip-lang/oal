use crate::errors::{Error, Result};
use crate::inference::{constrain, substitute, tag_type, TagSeq, TypeConstraint};
use crate::reduce;
use crate::scan::Scan;
use crate::scope::Env;
use crate::transform::Transform;
use enum_map::EnumMap;
use oal_syntax::ast;
use oal_syntax::ast::{Node, Tagged};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub enum UriSegment {
    Literal(ast::Literal),
    Variable(Prop),
}

impl<T: Node> TryFrom<&ast::UriSegment<T>> for UriSegment {
    type Error = Error;

    fn try_from(s: &ast::UriSegment<T>) -> Result<Self> {
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

impl<T: Node> TryFrom<&ast::Uri<T>> for Uri {
    type Error = Error;

    fn try_from(uri: &ast::Uri<T>) -> Result<Self> {
        let spec: Result<Vec<UriSegment>> = uri.spec.iter().map(|s| s.try_into()).collect();
        spec.map(|spec| Uri { spec })
    }
}

impl<T: Node> TryFrom<&ast::Expr<T>> for Uri {
    type Error = Error;

    fn try_from(e: &ast::Expr<T>) -> Result<Self> {
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

impl<T: Node> TryFrom<&ast::Array<T>> for Array {
    type Error = Error;

    fn try_from(a: &ast::Array<T>) -> Result<Self> {
        a.item.as_ref().as_ref().try_into().map(|item| Array {
            item: Box::new(item),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariadicOp {
    pub op: ast::Operator,
    pub schemas: Vec<Schema>,
}

impl<T: Node> TryFrom<&ast::VariadicOp<T>> for VariadicOp {
    type Error = Error;

    fn try_from(op: &ast::VariadicOp<T>) -> Result<Self> {
        let schemas: Result<Vec<_>> = op.exprs.iter().map(|e| e.as_ref().try_into()).collect();
        schemas.map(|schemas| VariadicOp { op: op.op, schemas })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Schema {
    Prim(ast::Primitive),
    Rel(Relation),
    Uri(Uri),
    Array(Array),
    Object(Object),
    Op(VariadicOp),
}

impl<T: Node> TryFrom<&ast::Expr<T>> for Schema {
    type Error = Error;

    fn try_from(e: &ast::Expr<T>) -> Result<Self> {
        match e {
            ast::Expr::Prim(prim) => Ok(Schema::Prim(*prim)),
            ast::Expr::Rel(rel) => rel.try_into().map(|r| Schema::Rel(r)),
            ast::Expr::Uri(uri) => uri.try_into().map(|u| Schema::Uri(u)),
            ast::Expr::Array(arr) => arr.try_into().map(|a| Schema::Array(a)),
            ast::Expr::Object(obj) => obj.try_into().map(|o| Schema::Object(o)),
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

impl<T: Node> TryFrom<&ast::Property<T>> for Prop {
    type Error = Error;

    fn try_from(p: &ast::Property<T>) -> Result<Self> {
        p.val.as_ref().try_into().map(|s| Prop {
            name: p.key.clone(),
            schema: s,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Object {
    pub props: Vec<Prop>,
}

impl<T: Node> TryFrom<&ast::Object<T>> for Object {
    type Error = Error;

    fn try_from(o: &ast::Object<T>) -> Result<Self> {
        let props: Result<Vec<_>> = o.props.iter().map(|p| p.try_into()).collect();
        props.map(|props| Object { props })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transfer {
    pub domain: Option<Box<Schema>>,
    pub range: Box<Schema>,
}

impl<T: Node> TryFrom<&ast::Transfer<T>> for Transfer {
    type Error = Error;

    fn try_from(xfer: &ast::Transfer<T>) -> Result<Self> {
        let range = xfer
            .range
            .as_ref()
            .as_ref()
            .try_into()
            .map(|r| Box::new(r))?;
        let domain = match &xfer.domain {
            Some(d) => d.as_ref().as_ref().try_into().map(|d| Some(Box::new(d))),
            None => Ok(None),
        }?;
        Ok(Transfer { range, domain })
    }
}

pub type Transfers = EnumMap<ast::Method, Option<Transfer>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Relation {
    pub uri: Uri,
    pub xfers: Transfers,
}

impl<T: Node> TryFrom<&ast::Relation<T>> for Relation {
    type Error = Error;

    fn try_from(r: &ast::Relation<T>) -> Result<Self> {
        let uri: Uri = r.uri.as_ref().as_ref().try_into()?;
        let xfers = r
            .xfers
            .iter()
            .map(|(m, x)| match x.as_ref() {
                Some(t) => t.try_into().map(|t| (m, Some(t))),
                None => Ok((m, None)),
            })
            .collect::<Result<_>>()?;
        Ok(Relation { uri, xfers })
    }
}

impl<T: Node> TryFrom<&ast::Expr<T>> for Relation {
    type Error = Error;

    fn try_from(e: &ast::Expr<T>) -> Result<Self> {
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

impl<T: Node> TryFrom<&ast::Program<T>> for Spec {
    type Error = Error;

    fn try_from(prg: &ast::Program<T>) -> Result<Self> {
        let mut rels: Relations = HashMap::new();

        prg.stmts.iter().try_for_each(|stmt| match stmt {
            ast::Statement::Res(res) => {
                let rel = Relation::try_from(res.rel.as_ref());
                rel.and_then(|rel| match rels.entry(rel.uri.pattern()) {
                    Entry::Vacant(v) => {
                        v.insert(rel);
                        Ok(())
                    }
                    Entry::Occupied(_) => Err(Error::new("relation conflict")),
                })
            }
            _ => Ok(()),
        })?;

        Ok(Spec { rels })
    }
}

pub fn evaluate<T: Node + Tagged>(mut prg: ast::Program<T>) -> Result<Spec> {
    prg.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)?;

    let constraint = &mut TypeConstraint::new();

    prg.scan(constraint, &mut Env::new(), constrain)?;

    let subst = &mut constraint.unify()?;

    prg.transform(subst, &mut Env::new(), substitute)?;

    prg.transform(&mut (), &mut Env::new(), reduce)?;

    Spec::try_from(&prg)
}
