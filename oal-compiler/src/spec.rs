use crate::annotation::Annotated;
use crate::errors::{Error, Kind, Result};
use enum_map::EnumMap;
use indexmap::IndexMap;
use oal_syntax::ast;
use oal_syntax::ast::AsExpr;
use std::fmt::Debug;
use std::num::NonZeroU16;

#[derive(Clone, Debug, PartialEq)]
pub enum UriSegment {
    Literal(ast::Literal),
    Variable(Box<Property>),
}

impl<T> TryFrom<&ast::UriSegment<T>> for UriSegment
where
    T: AsExpr + Annotated,
{
    type Error = Error;

    fn try_from(s: &ast::UriSegment<T>) -> Result<Self> {
        match s {
            ast::UriSegment::Literal(l) => Ok(UriSegment::Literal(l.clone())),
            ast::UriSegment::Variable(p) => {
                Property::try_from(p).map(|p| UriSegment::Variable(Box::new(p)))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Uri {
    pub path: Vec<UriSegment>,
    pub params: Option<Object>,
}

impl Uri {
    pub fn pattern(&self) -> String {
        self.path
            .iter()
            .map(|s| match s {
                UriSegment::Literal(l) => format!("/{}", l),
                UriSegment::Variable(t) => format!("/{{{}}}", t.name),
            })
            .collect()
    }

    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Uri(uri) = e.as_node().as_expr() {
            let path = uri
                .path
                .iter()
                .map(UriSegment::try_from)
                .collect::<Result<Vec<UriSegment>>>()?;
            let params = if let Some(p) = &uri.params {
                let obj = Object::try_from(p.as_ref())?;
                Some(obj)
            } else {
                None
            };
            Ok(Uri { path, params })
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
pub struct Property {
    pub name: ast::Ident,
    pub schema: Schema,
    pub desc: Option<String>,
}

impl Property {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Property(prop) = e.as_node().as_expr() {
            let desc = e.annotation().and_then(|a| a.get_string("description"));
            Schema::try_from(prop.val.as_ref()).map(|s| Property {
                name: prop.name.clone(),
                schema: s,
                desc,
            })
        } else {
            Err(Error::new(Kind::UnexpectedExpression, "not a property").with(e))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Object {
    pub props: Vec<Property>,
}

impl Object {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Object(o) = e.as_node().as_expr() {
            let props: Result<Vec<_>> = o.props.iter().map(Property::try_from).collect();
            props.map(|props| Object { props })
        } else {
            Err(Error::new(Kind::UnexpectedExpression, "not an object").with(e))
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct HttpStatus(NonZeroU16);

impl From<HttpStatus> for u16 {
    fn from(s: HttpStatus) -> Self {
        s.0.into()
    }
}

impl TryFrom<u16> for HttpStatus {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self> {
        if value >= 100 && value <= 599 {
            let status = unsafe { NonZeroU16::new_unchecked(value) };
            Ok(HttpStatus(status))
        } else {
            Err(Error::new(Kind::InvalidHttpStatus, "not in range").with(&value))
        }
    }
}

pub type MediaType = String;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Content {
    pub schema: Option<Box<Schema>>,
    pub status: Option<HttpStatus>,
    pub media: Option<MediaType>,
    pub desc: Option<String>,
}

impl From<Schema> for Content {
    fn from(s: Schema) -> Self {
        let desc = s.desc.clone();
        let schema = Some(s.into());
        let status = None;
        let media = None;
        Content {
            schema,
            status,
            media,
            desc,
        }
    }
}

impl Content {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Content(c) = e.as_node().as_expr() {
            let schema = match &c.schema {
                Some(s) => Schema::try_from(s.as_ref()).map(|s| Some(Box::new(s))),
                None => Ok(None),
            }?;
            let status = match c.status {
                Some(s) => HttpStatus::try_from(s).map(Some),
                None => Ok(None),
            }?;
            let media = c.media.clone();
            let desc = e.annotation().and_then(|a| a.get_string("description"));
            Ok(Content {
                schema,
                status,
                media,
                desc,
            })
        } else {
            Schema::try_from(e).map(Content::from)
        }
    }
}

pub type Ranges = IndexMap<(Option<HttpStatus>, Option<MediaType>), Content>;

#[derive(Clone, Debug, PartialEq)]
pub struct Transfer {
    pub methods: EnumMap<ast::Method, bool>,
    pub domain: Content,
    pub ranges: Ranges,
    pub params: Option<Object>,
    pub summary: Option<String>,
}

impl Transfer {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        if let ast::Expr::Xfer(xfer) = e.as_node().as_expr() {
            let methods = xfer.methods;
            let ranges = xfer
                .ranges
                .iter()
                .map(|r| Content::try_from(r).map(|c| ((c.status, c.media.clone()), c)))
                .collect::<Result<_>>()?;
            let domain = match &xfer.domain {
                Some(d) => Content::try_from(d.as_ref()),
                None => Ok(Content::default()),
            }?;
            let params = match &xfer.params {
                Some(x) => Object::try_from(x.as_ref()).map(Some),
                None => Ok(None),
            }?;
            let summary = e.annotation().and_then(|a| a.get_string("summary"));
            Ok(Transfer {
                methods,
                domain,
                ranges,
                params,
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
