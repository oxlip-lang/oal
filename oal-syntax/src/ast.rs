use crate::{Pair, Rule};
use enum_map::{Enum, EnumMap};
use std::fmt::Debug;
use std::iter::{empty, once, Flatten, Once};
use std::rc::Rc;
use std::slice::{Iter, IterMut};

pub type Literal = Rc<str>;
pub type Ident = Rc<str>;

#[derive(Debug)]
pub enum NodeRef<'a, T> {
    Expr(&'a T),
    Decl(&'a Declaration<T>),
    Res(&'a Resource<T>),
    Ann(&'a Annotation),
    Use(&'a Import),
}

#[derive(Debug)]
pub enum NodeMut<'a, T> {
    Expr(&'a mut T),
    Decl(&'a mut Declaration<T>),
    Res(&'a mut Resource<T>),
    Ann(&'a mut Annotation),
    Use(&'a mut Import),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<T> {
    Prim(Primitive),
    Rel(Relation<T>),
    Uri(Uri<T>),
    Array(Array<T>),
    Property(Property<T>),
    Object(Object<T>),
    Content(Content<T>),
    Xfer(Transfer<T>),
    Op(VariadicOp<T>),
    Var(Ident),
    Lambda(Lambda<T>),
    App(Application<T>),
    Binding(Ident),
}

impl<T> Expr<T> {
    pub fn into_node(self) -> NodeExpr<T> {
        NodeExpr {
            inner: self,
            ann: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodeExpr<T> {
    pub inner: Expr<T>,
    pub ann: Option<Annotation>,
}

impl<T> NodeExpr<T> {
    pub fn as_expr(&self) -> &Expr<T> {
        &self.inner
    }

    pub fn as_expr_mut(&mut self) -> &mut Expr<T> {
        &mut self.inner
    }
}

pub trait AsRefNode {
    fn as_node(&self) -> &NodeExpr<Self>
    where
        Self: Sized;
}

pub trait AsMutNode {
    fn as_node_mut(&mut self) -> &mut NodeExpr<Self>
    where
        Self: Sized;
}

pub trait AsExpr: From<NodeExpr<Self>> + AsRefNode + AsMutNode + Clone + Debug {}

impl<T> AsExpr for T where T: From<NodeExpr<T>> + AsRefNode + AsMutNode + Clone + Debug {}

pub trait FromPair: Sized {
    fn from_pair(_: Pair<'_>) -> Self;
}

pub trait IntoExpr<T>: Sized {
    fn into_expr(self) -> T;
}

impl<T: FromPair> IntoExpr<T> for Pair<'_> {
    fn into_expr(self) -> T {
        T::from_pair(self)
    }
}

impl<T: AsExpr> FromPair for T {
    fn from_pair(p: Pair) -> T {
        match p.as_rule() {
            Rule::expr_type | Rule::paren_type | Rule::app_type | Rule::xfer_type => {
                p.into_inner().next().unwrap().into_expr()
            }
            Rule::term_type => {
                let mut inner = p.into_inner();
                let mut term: T = inner.next().unwrap().into_expr();
                term.as_node_mut().ann = inner.next().map(Annotation::from_pair);
                term
            }
            Rule::prim_type => Expr::Prim(p.into_expr()).into_node().into(),
            Rule::rel_type => Expr::Rel(p.into_expr()).into_node().into(),
            Rule::uri_type => Expr::Uri(p.into_expr()).into_node().into(),
            Rule::array_type => Expr::Array(p.into_expr()).into_node().into(),
            Rule::prop_type => Expr::Property(p.into_expr()).into_node().into(),
            Rule::object_type => Expr::Object(p.into_expr()).into_node().into(),
            Rule::content_type => Expr::Content(p.into_expr()).into_node().into(),
            Rule::var => Expr::Var(p.as_str().into()).into_node().into(),
            Rule::binding => Expr::Binding(p.as_str().into()).into_node().into(),
            Rule::join_type | Rule::any_type | Rule::sum_type | Rule::range_type => {
                let mut op = VariadicOp::from_pair(p);
                if op.exprs.len() == 1 {
                    op.exprs.remove(0)
                } else {
                    Expr::Op(op).into_node().into()
                }
            }
            Rule::apply => Expr::App(p.into_expr()).into_node().into(),
            Rule::xfer => Expr::Xfer(p.into_expr()).into_node().into(),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program<T> {
    pub stmts: Vec<Statement<T>>,
}

impl<T: AsExpr> FromPair for Program<T> {
    fn from_pair(p: Pair) -> Self {
        let stmts = p
            .into_inner()
            .flat_map(|p| match p.as_rule() {
                Rule::stmt => Some(p.into_expr()),
                _ => None,
            })
            .collect();
        Program { stmts }
    }
}

impl<'a, T> IntoIterator for &'a Program<T> {
    type Item = &'a Statement<T>;
    type IntoIter = Iter<'a, Statement<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.stmts.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Program<T> {
    type Item = &'a mut Statement<T>;
    type IntoIter = IterMut<'a, Statement<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.stmts.iter_mut()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Declaration<T> {
    pub name: Ident,
    pub expr: T,
}

impl<T: AsExpr> FromPair for Declaration<T> {
    fn from_pair(p: Pair) -> Self {
        let mut p = p.into_inner();
        let name = p.nth(1).unwrap().as_str().into();
        let bindings: Vec<T> = p
            .next()
            .unwrap()
            .into_inner()
            .map(|p| p.into_expr())
            .collect();
        let expr = p.next().unwrap().into_expr();
        let expr = if bindings.is_empty() {
            expr
        } else {
            Expr::Lambda(Lambda {
                bindings,
                body: Box::new(expr),
            })
            .into_node()
            .into()
        };
        Declaration { name, expr }
    }
}

impl<'a, T> IntoIterator for &'a Declaration<T> {
    type Item = &'a T;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(&self.expr)
    }
}

impl<'a, T> IntoIterator for &'a mut Declaration<T> {
    type Item = &'a mut T;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(&mut self.expr)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Resource<T> {
    pub rel: T,
}

impl<T: AsExpr> FromPair for Resource<T> {
    fn from_pair(p: Pair) -> Self {
        Resource {
            rel: p.into_inner().nth(1).unwrap().into_expr(),
        }
    }
}

impl<'a, T> IntoIterator for &'a Resource<T> {
    type Item = &'a T;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(&self.rel)
    }
}

impl<'a, T> IntoIterator for &'a mut Resource<T> {
    type Item = &'a mut T;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(&mut self.rel)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Annotation {
    pub text: String,
}

impl FromPair for Annotation {
    fn from_pair(p: Pair) -> Self {
        Annotation {
            text: p.into_inner().next().unwrap().as_str().to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    pub module: String,
}

impl FromPair for Import {
    fn from_pair(p: Pair) -> Self {
        let module = p
            .into_inner()
            .nth(1)
            .unwrap()
            .into_inner()
            .next()
            .unwrap()
            .as_str()
            .to_owned();
        Import { module }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<T> {
    Decl(Declaration<T>),
    Res(Resource<T>),
    Ann(Annotation),
    Use(Import),
}

impl<T: AsExpr> FromPair for Statement<T> {
    fn from_pair(p: Pair) -> Self {
        let p = p.into_inner().next().unwrap();
        match p.as_rule() {
            Rule::decl => Statement::Decl(p.into_expr()),
            Rule::res => Statement::Res(p.into_expr()),
            Rule::ann => Statement::Ann(p.into_expr()),
            Rule::import => Statement::Use(p.into_expr()),
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Enum)]
pub enum Method {
    Get,
    Put,
    Post,
    Patch,
    Delete,
    Options,
    Head,
}

impl FromPair for Method {
    fn from_pair(p: Pair) -> Self {
        match p.into_inner().next().unwrap().as_rule() {
            Rule::get_kw => Method::Get,
            Rule::put_kw => Method::Put,
            Rule::post_kw => Method::Post,
            Rule::patch_kw => Method::Patch,
            Rule::delete_kw => Method::Delete,
            Rule::options_kw => Method::Options,
            Rule::head_kw => Method::Head,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transfer<T> {
    pub methods: EnumMap<Method, bool>,
    pub domain: Option<Box<T>>,
    pub ranges: Box<T>,
    pub params: Option<Box<T>>,
}

impl<T: AsExpr> FromPair for Transfer<T> {
    fn from_pair(p: Pair) -> Self {
        let mut inner = p.into_inner();

        let methods: EnumMap<_, _> = inner
            .next()
            .unwrap()
            .into_inner()
            .map(|p| (p.into_expr(), true))
            .collect();

        let params: Option<Box<T>> = inner
            .next()
            .unwrap()
            .into_inner()
            .next()
            .map(|p| Box::new(p.into_expr()));

        let domain = inner
            .next()
            .unwrap()
            .into_inner()
            .next()
            .map(|p| Box::new(p.into_expr()));

        let ranges = T::from_pair(inner.next().unwrap()).into();

        Transfer {
            methods,
            domain,
            ranges,
            params,
        }
    }
}

impl<'a, T> IntoIterator for &'a Transfer<T> {
    type Item = &'a T;
    type IntoIter = Flatten<std::array::IntoIter<Option<Self::Item>, 3>>;

    fn into_iter(self) -> Self::IntoIter {
        [
            Some(self.ranges.as_ref()),
            self.domain.as_ref().map(AsRef::as_ref),
            self.params.as_ref().map(AsRef::as_ref),
        ]
        .into_iter()
        .flatten()
    }
}

impl<'a, T> IntoIterator for &'a mut Transfer<T> {
    type Item = &'a mut T;
    type IntoIter = Flatten<std::array::IntoIter<Option<Self::Item>, 3>>;

    fn into_iter(self) -> Self::IntoIter {
        [
            Some(self.ranges.as_mut()),
            self.domain.as_mut().map(AsMut::as_mut),
            self.params.as_mut().map(AsMut::as_mut),
        ]
        .into_iter()
        .flatten()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Relation<T> {
    pub uri: Box<T>,
    pub xfers: Vec<T>,
}

impl<T: AsExpr> FromPair for Relation<T> {
    fn from_pair(p: Pair) -> Self {
        let mut inner = p.into_inner();

        let uri: Box<_> = T::from_pair(inner.next().unwrap()).into();

        let xfers: Vec<_> = inner.map(IntoExpr::into_expr).collect();

        Relation { uri, xfers }
    }
}

impl<'a, T> IntoIterator for &'a Relation<T> {
    type Item = &'a T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(once(self.uri.as_ref()).chain(self.xfers.iter()))
    }
}

impl<'a, T> IntoIterator for &'a mut Relation<T> {
    type Item = &'a mut T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(once(self.uri.as_mut()).chain(self.xfers.iter_mut()))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UriSegment<T> {
    Literal(Literal),
    Variable(T),
}

impl<T> UriSegment<T> {
    pub fn root() -> Self {
        UriSegment::Literal("".into())
    }
}

impl<T: AsExpr> FromPair for UriSegment<T> {
    fn from_pair(p: Pair) -> Self {
        match p.as_rule() {
            Rule::uri_var => UriSegment::Variable(p.into_inner().next().unwrap().into_expr()),
            Rule::uri_literal => UriSegment::Literal(p.as_str().into()),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Uri<T> {
    pub path: Vec<UriSegment<T>>,
    pub params: Option<Box<T>>,
}

impl<'a, T> IntoIterator for &'a Uri<T> {
    type Item = &'a T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        let path = self.path.iter().filter_map(|s| {
            if let UriSegment::Variable(t) = s {
                Some(t)
            } else {
                None
            }
        });
        if let Some(params) = &self.params {
            Box::new(path.chain(once(params.as_ref())))
        } else {
            Box::new(path)
        }
    }
}

impl<'a, T> IntoIterator for &'a mut Uri<T> {
    type Item = &'a mut T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        let path = self.path.iter_mut().filter_map(|s| {
            if let UriSegment::Variable(t) = s {
                Some(t)
            } else {
                None
            }
        });
        if let Some(params) = &mut self.params {
            Box::new(path.chain(once(params.as_mut())))
        } else {
            Box::new(path)
        }
    }
}

impl<T: AsExpr> FromPair for Uri<T> {
    fn from_pair(p: Pair) -> Self {
        let p = p.into_inner().next().unwrap();
        let (path, params) = match p.as_rule() {
            Rule::uri_kw => Default::default(),
            Rule::uri_root => (vec![UriSegment::root()], None),
            Rule::uri_template => {
                let mut inner = p.into_inner();
                let path = inner
                    .next()
                    .unwrap()
                    .into_inner()
                    .map(UriSegment::from_pair)
                    .collect();
                let params = inner
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .map(|p| Box::new(p.into_expr()));
                (path, params)
            }
            _ => unreachable!(),
        };
        Uri { path, params }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Array<T> {
    pub item: Box<T>,
}

impl<T: AsExpr> FromPair for Array<T> {
    fn from_pair(p: Pair) -> Self {
        let item = Box::new(p.into_inner().next().unwrap().into_expr());
        Array { item }
    }
}

impl<'a, T> IntoIterator for &'a Array<T> {
    type Item = &'a T;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(self.item.as_ref())
    }
}

impl<'a, T> IntoIterator for &'a mut Array<T> {
    type Item = &'a mut T;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(self.item.as_mut())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Property<T> {
    pub name: Ident,
    pub val: Box<T>,
}

impl<T: AsExpr> FromPair for Property<T> {
    fn from_pair(p: Pair) -> Self {
        let mut inner = p.into_inner();
        let name = inner
            .next()
            .unwrap()
            .into_inner()
            .next()
            .unwrap()
            .as_str()
            .into();
        let val = Box::new(inner.next().unwrap().into_expr());
        Property { name, val }
    }
}

impl<'a, T> IntoIterator for &'a Property<T> {
    type Item = &'a T;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(self.val.as_ref())
    }
}

impl<'a, T> IntoIterator for &'a mut Property<T> {
    type Item = &'a mut T;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(self.val.as_mut())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Object<T> {
    pub props: Vec<T>,
}

impl<T> Default for Object<T> {
    fn default() -> Self {
        Object { props: Vec::new() }
    }
}

impl<T: AsExpr> FromPair for Object<T> {
    fn from_pair(p: Pair) -> Self {
        let props = p.into_inner().map(|p| p.into_expr()).collect();
        Object { props }
    }
}

impl<'a, T> IntoIterator for &'a Object<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.props.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Object<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.props.iter_mut()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Content<T> {
    pub schema: Option<Box<T>>,
    pub status: Option<u16>,
    pub media: Option<String>,
}

impl<T: AsExpr> FromPair for Content<T> {
    fn from_pair(p: Pair) -> Self {
        let (mut schema, mut status, mut media) = (None, None, None);
        for p in p.into_inner() {
            match p.as_rule() {
                Rule::http_status => status = Some(p.as_str().parse().unwrap()),
                Rule::http_media_range => media = Some(p.as_str().to_owned()),
                Rule::expr_type => schema = Some(Box::new(p.into_expr())),
                _ => unreachable!(),
            }
        }
        Content {
            schema,
            status,
            media,
        }
    }
}

impl<'a, T> IntoIterator for &'a Content<T> {
    type Item = &'a T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match &self.schema {
            None => Box::new(empty()),
            Some(s) => Box::new(once(s.as_ref())),
        }
    }
}

impl<'a, T> IntoIterator for &'a mut Content<T> {
    type Item = &'a mut T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match &mut self.schema {
            None => Box::new(empty()),
            Some(s) => Box::new(once(s.as_mut())),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Operator {
    Join,
    Any,
    Sum,
    Range,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariadicOp<T> {
    pub op: Operator,
    pub exprs: Vec<T>,
}

impl<T: AsExpr> FromPair for VariadicOp<T> {
    fn from_pair(p: Pair) -> Self {
        let op = match p.as_rule() {
            Rule::join_type => Operator::Join,
            Rule::any_type => Operator::Any,
            Rule::sum_type => Operator::Sum,
            Rule::range_type => Operator::Range,
            _ => unreachable!(),
        };
        let exprs = p.into_inner().map(|p| p.into_expr()).collect();
        VariadicOp { op, exprs }
    }
}

impl<'a, T: AsExpr> IntoIterator for &'a VariadicOp<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.exprs.iter()
    }
}

impl<'a, T: AsExpr> IntoIterator for &'a mut VariadicOp<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.exprs.iter_mut()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Primitive {
    Num,
    Str,
    Bool,
}

impl FromPair for Primitive {
    fn from_pair(p: Pair) -> Self {
        match p.into_inner().next().unwrap().as_rule() {
            Rule::num_kw => Primitive::Num,
            Rule::str_kw => Primitive::Str,
            Rule::bool_kw => Primitive::Bool,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lambda<T> {
    pub bindings: Vec<T>,
    pub body: Box<T>,
}

impl<'a, T> IntoIterator for &'a Lambda<T> {
    type Item = &'a T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.bindings.iter().chain(once(self.body.as_ref())))
    }
}

impl<'a, T> IntoIterator for &'a mut Lambda<T> {
    type Item = &'a mut T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.bindings.iter_mut().chain(once(self.body.as_mut())))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Application<T> {
    pub name: Ident,
    pub args: Vec<T>,
}

impl<T: AsExpr> FromPair for Application<T> {
    fn from_pair(p: Pair) -> Self {
        let mut inner = p.into_inner();
        let name = inner.next().unwrap().as_str().into();
        let args = inner.into_iter().map(|p| p.into_expr()).collect();
        Application { name, args }
    }
}

impl<'a, T> IntoIterator for &'a Application<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Application<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.iter_mut()
    }
}
