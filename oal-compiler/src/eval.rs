use crate::annotation::{annotate, Annotated};
use crate::errors::{Error, Kind, Result};
use crate::inference::{constrain, substitute, tag_type, InferenceSet, TagSeq};
use crate::reduction::reduce;
use crate::scan::Scan;
use crate::scope::Env;
use crate::tag::Tagged;
use crate::transform::Transform;
use crate::typecheck::type_check;
use enum_map::EnumMap;
use oal_syntax::ast;
use oal_syntax::ast::AsExpr;
use serde_yaml::Value;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub enum UriSegment {
    Literal(ast::Literal),
    Variable(Prop),
}

impl<T> TryFrom<&ast::UriSegment<T>> for UriSegment
where
    T: AsExpr + Annotated,
{
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

    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        match e.as_ref() {
            ast::Expr::Uri(uri) => uri.try_into(),
            _ => Err(Error::new(Kind::UnexpectedExpression, "not a URI").with(e)),
        }
    }
}

impl<T> TryFrom<&ast::Uri<T>> for Uri
where
    T: AsExpr + Annotated,
{
    type Error = Error;

    fn try_from(uri: &ast::Uri<T>) -> Result<Self> {
        let spec: Result<Vec<UriSegment>> = uri.spec.iter().map(|s| s.try_into()).collect();
        spec.map(|spec| Uri { spec })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Array {
    pub item: Box<Schema>,
}

impl<T> TryFrom<&ast::Array<T>> for Array
where
    T: AsExpr + Annotated,
{
    type Error = Error;

    fn try_from(a: &ast::Array<T>) -> Result<Self> {
        Schema::try_from(a.item.as_ref()).map(|item| Array {
            item: Box::new(item),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariadicOp {
    pub op: ast::Operator,
    pub schemas: Vec<Schema>,
}

impl<T> TryFrom<&ast::VariadicOp<T>> for VariadicOp
where
    T: AsExpr + Annotated,
{
    type Error = Error;

    fn try_from(op: &ast::VariadicOp<T>) -> Result<Self> {
        let schemas: Result<Vec<_>> = op.exprs.iter().map(Schema::try_from).collect();
        schemas.map(|schemas| VariadicOp { op: op.op, schemas })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    pub expr: Expr,
    pub desc: Option<String>,
}

impl Schema {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        let expr = match e.as_ref() {
            ast::Expr::Prim(prim) => Ok(Expr::Prim(*prim)),
            ast::Expr::Rel(rel) => rel.try_into().map(|r| Expr::Rel(r)),
            ast::Expr::Uri(uri) => uri.try_into().map(|u| Expr::Uri(u)),
            ast::Expr::Array(arr) => arr.try_into().map(|a| Expr::Array(a)),
            ast::Expr::Object(obj) => obj.try_into().map(|o| Expr::Object(o)),
            ast::Expr::Op(op) => op.try_into().map(|o| Expr::Op(o)),
            _ => Err(Error::new(Kind::UnexpectedExpression, "expected schema-like").with(e)),
        }?;
        let desc = e
            .annotation()
            .and_then(|a| a.props.get(&Value::String("description".to_owned())))
            .and_then(|a| a.as_str())
            .map(|a| a.to_owned());
        Ok(Schema { expr, desc })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Prim(ast::Primitive),
    Rel(Relation),
    Uri(Uri),
    Array(Array),
    Object(Object),
    Op(VariadicOp),
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

impl<T> TryFrom<&ast::Object<T>> for Object
where
    T: AsExpr + Annotated,
{
    type Error = Error;

    fn try_from(o: &ast::Object<T>) -> Result<Self> {
        let props: Result<Vec<_>> = o.props.iter().map(|p| p.try_into()).collect();
        props.map(|props| Object { props })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Content {
    pub schema: Option<Box<Schema>>,
}

impl From<Schema> for Content {
    fn from(s: Schema) -> Self {
        Content {
            schema: Some(s.into()),
        }
    }
}

impl Content {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        match e.as_ref() {
            ast::Expr::Content(c) => c.try_into(),
            _ => Schema::try_from(e).map(Content::from),
        }
    }
}

impl<T> TryFrom<&ast::Content<T>> for Content
where
    T: AsExpr + Annotated,
{
    type Error = Error;

    fn try_from(c: &ast::Content<T>) -> Result<Self> {
        let schema = match &c.schema {
            Some(s) => Schema::try_from(s.as_ref()).map(|s| Some(Box::new(s))),
            None => Ok(None),
        }?;
        Ok(Content { schema })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transfer {
    pub methods: EnumMap<ast::Method, bool>,
    pub domain: Content,
    pub range: Content,
}

impl Transfer {
    fn try_from<T: AsExpr + Annotated>(e: &T) -> Result<Self> {
        match e.as_ref() {
            ast::Expr::Xfer(xfer) => xfer.try_into(),
            _ => Err(Error::new(Kind::UnexpectedExpression, "not a transfer").with(e)),
        }
    }
}

impl<T> TryFrom<&ast::Transfer<T>> for Transfer
where
    T: AsExpr + Annotated,
{
    type Error = Error;

    fn try_from(xfer: &ast::Transfer<T>) -> Result<Self> {
        let methods = xfer.methods.clone();
        let range = Content::try_from(xfer.range.as_ref())?;
        let domain = match &xfer.domain {
            Some(d) => Content::try_from(d.as_ref()),
            None => Ok(Content { schema: None }),
        }?;
        Ok(Transfer {
            methods,
            range,
            domain,
        })
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
        match e.as_ref() {
            ast::Expr::Rel(rel) => rel.try_into(),
            _ => Err(Error::new(Kind::UnexpectedExpression, "not a relation").with(e)),
        }
    }
}

impl<T> TryFrom<&ast::Relation<T>> for Relation
where
    T: AsExpr + Annotated,
{
    type Error = Error;

    fn try_from(r: &ast::Relation<T>) -> Result<Self> {
        let uri = Uri::try_from(r.uri.as_ref())?;
        let mut xfers = Transfers::default();
        for x in r.xfers.iter() {
            let t = Transfer::try_from(x)?;
            for (m, b) in t.methods {
                if b {
                    xfers[m] = Some(t.clone());
                }
            }
        }
        Ok(Relation { uri, xfers })
    }
}

pub type PathPattern = String;
pub type Relations = HashMap<PathPattern, Relation>;

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
        let mut rels: Relations = HashMap::new();

        prg.stmts.iter().try_for_each(|stmt| match stmt {
            ast::Statement::Res(res) => {
                let rel = Relation::try_from(&res.rel);
                rel.and_then(|rel| match rels.entry(rel.uri.pattern()) {
                    Entry::Vacant(v) => {
                        v.insert(rel);
                        Ok(())
                    }
                    Entry::Occupied(_) => Err(Error::new(Kind::RelationConflict, "").with(&rel)),
                })
            }
            _ => Ok(()),
        })?;

        Ok(Spec { rels })
    }
}

pub fn evaluate<T>(mut prg: ast::Program<T>) -> Result<Spec>
where
    T: AsExpr + Tagged + Annotated,
{
    prg.transform(&mut TagSeq::new(), &mut Env::new(), &mut tag_type)?;

    let constraint = &mut InferenceSet::new();

    prg.scan(constraint, &mut Env::new(), &mut constrain)?;

    let subst = &mut constraint.unify()?;

    prg.transform(subst, &mut Env::new(), &mut substitute)?;

    prg.scan(&mut (), &mut Env::new(), &mut type_check)?;

    prg.transform(&mut None, &mut Env::new(), &mut annotate)?;

    prg.transform(&mut (), &mut Env::new(), &mut reduce)?;

    Spec::try_from(&prg)
}
