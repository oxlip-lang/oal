use crate::annotation::Annotated;
use crate::errors::{Error, Kind, Result};
use enum_map::EnumMap;
use indexmap::IndexMap;
use oal_syntax::ast;
use oal_syntax::ast::AsExpr;
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub enum UriSegment {
    Literal(ast::Literal),
    Variable(Box<Prop>),
}

impl<T> TryFrom<&ast::UriSegment<T>> for UriSegment
where
    T: AsExpr + Annotated,
{
    type Error = Error;

    fn try_from(s: &ast::UriSegment<T>) -> Result<Self> {
        match s {
            ast::UriSegment::Literal(l) => Ok(UriSegment::Literal(l.clone())),
            ast::UriSegment::Variable(p) => p.try_into().map(|p| UriSegment::Variable(Box::new(p))),
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

    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Uri(uri) = e.as_node().as_expr() {
            let spec: Result<Vec<UriSegment>> = uri.spec.iter().map(|s| s.try_into()).collect();
            spec.map(|spec| Uri { spec })
        } else {
            Err(Error::new(Kind::UnexpectedExpression, "not a URI").with(e))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Array {
    pub item: Schema,
}

impl Array {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Array(a) = e.as_node().as_expr() {
            Schema::try_from(a.item.as_ref()).map(|item| Array { item })
        } else {
            Err(Error::new(Kind::UnexpectedExpression, "not an array").with(e))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariadicOp {
    pub op: ast::Operator,
    pub schemas: Vec<Schema>,
}

impl VariadicOp {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Op(op) = e.as_node().as_expr() {
            let schemas: Result<Vec<_>> = op.exprs.iter().map(Schema::try_from).collect();
            schemas.map(|schemas| VariadicOp { op: op.op, schemas })
        } else {
            Err(Error::new(Kind::UnexpectedExpression, "not an operation").with(e))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    pub expr: Expr,
    pub desc: Option<String>,
    pub title: Option<String>,
}

impl Schema {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        let expr = Expr::try_from(e)?;
        let desc = e.annotation().and_then(|a| a.get_string("description"));
        let title = e.annotation().and_then(|a| a.get_string("title"));
        Ok(Schema { expr, desc, title })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Prim(ast::Primitive),
    Rel(Box<Relation>),
    Uri(Uri),
    Array(Box<Array>),
    Object(Object),
    Op(VariadicOp),
}

impl Expr {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        match e.as_node().as_expr() {
            ast::Expr::Prim(prim) => Ok(Expr::Prim(*prim)),
            ast::Expr::Rel(_) => Relation::try_from(e).map(|r| Expr::Rel(Box::new(r))),
            ast::Expr::Uri(_) => Uri::try_from(e).map(Expr::Uri),
            ast::Expr::Array(_) => Array::try_from(e).map(|a| Expr::Array(Box::new(a))),
            ast::Expr::Object(_) => Object::try_from(e).map(Expr::Object),
            ast::Expr::Op(_) => VariadicOp::try_from(e).map(Expr::Op),
            _ => Err(Error::new(Kind::UnexpectedExpression, "expected schema-like").with(e)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prop {
    pub name: ast::Ident,
    pub schema: Schema,
}

impl<T> TryFrom<&ast::Property<T>> for Prop
where
    T: AsExpr + Annotated,
{
    type Error = Error;

    fn try_from(p: &ast::Property<T>) -> Result<Self> {
        Schema::try_from(&p.val).map(|s| Prop {
            name: p.key.clone(),
            schema: s,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Object {
    pub props: Vec<Prop>,
}

impl Object {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Object(o) = e.as_node().as_expr() {
            let props: Result<Vec<_>> = o.props.iter().map(|p| p.try_into()).collect();
            props.map(|props| Object { props })
        } else {
            Err(Error::new(Kind::UnexpectedExpression, "not an object").with(e))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Content {
    pub schema: Option<Box<Schema>>,
    pub desc: Option<String>,
}

impl From<Schema> for Content {
    fn from(s: Schema) -> Self {
        let desc = s.desc.clone();
        let schema = Some(s.into());
        Content { schema, desc }
    }
}

impl Content {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Content(c) = e.as_node().as_expr() {
            let schema = match &c.schema {
                Some(s) => Schema::try_from(s.as_ref()).map(|s| Some(Box::new(s))),
                None => Ok(None),
            }?;
            let desc = e.annotation().and_then(|a| a.get_string("description"));
            Ok(Content { schema, desc })
        } else {
            Schema::try_from(e).map(Content::from)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transfer {
    pub methods: EnumMap<ast::Method, bool>,
    pub domain: Content,
    pub range: Content,
    pub summary: Option<String>,
}

impl Transfer {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Xfer(xfer) = e.as_node().as_expr() {
            let methods = xfer.methods;
            let range = Content::try_from(xfer.range.as_ref())?;
            let domain = match &xfer.domain {
                Some(d) => Content::try_from(d.as_ref()),
                None => Ok(Content::default()),
            }?;
            let summary = e.annotation().and_then(|a| a.get_string("summary"));
            Ok(Transfer {
                methods,
                range,
                domain,
                summary,
            })
        } else {
            Err(Error::new(Kind::UnexpectedExpression, "not a transfer").with(e))
        }
    }
}

pub type Transfers = EnumMap<ast::Method, Option<Transfer>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Relation {
    pub uri: Uri,
    pub xfers: Transfers,
}

impl Relation {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Rel(rel) = e.as_node().as_expr() {
            let uri = Uri::try_from(rel.uri.as_ref())?;
            let mut xfers = Transfers::default();
            for x in rel.xfers.iter() {
                let t = Transfer::try_from(x)?;
                for (m, b) in t.methods {
                    if b {
                        xfers[m] = Some(t.clone());
                    }
                }
            }
            Ok(Relation { uri, xfers })
        } else {
            Err(Error::new(Kind::UnexpectedExpression, "not a relation").with(e))
        }
    }
}

pub type PathPattern = String;
pub type Relations = IndexMap<PathPattern, Relation>;

#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    pub rels: Relations,
}

impl<T> TryFrom<&ast::Program<T>> for Spec
where
    T: AsExpr + Annotated,
{
    type Error = Error;

    fn try_from(prg: &ast::Program<T>) -> Result<Self> {
        let mut rels: Relations = IndexMap::new();

        prg.stmts.iter().try_for_each(|stmt| match stmt {
            ast::Statement::Res(res) => {
                let rel = Relation::try_from(&res.rel);
                rel.and_then(|rel| match rels.entry(rel.uri.pattern()) {
                    indexmap::map::Entry::Vacant(v) => {
                        v.insert(rel);
                        Ok(())
                    }
                    indexmap::map::Entry::Occupied(_) => {
                        Err(Error::new(Kind::RelationConflict, "").with(&rel))
                    }
                })
            }
            _ => Ok(()),
        })?;

        Ok(Spec { rels })
    }
}
