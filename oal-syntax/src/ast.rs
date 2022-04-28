use crate::{Pair, Rule};
use enum_map::{Enum, EnumMap};
use std::fmt::Debug;
use std::iter::{once, Once};
use std::rc::Rc;
use std::slice::{Iter, IterMut};

pub type Literal = Rc<str>;
pub type Ident = Rc<str>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<T> {
    Prim(Primitive),
    Rel(Relation<T>),
    Uri(Uri<T>),
    Array(Array<T>),
    Object(Object<T>),
    Op(VariadicOp<T>),
    Var(Ident),
    Lambda(Lambda<T>),
    App(Application<T>),
    Binding(Ident),
    Ann(String),
}

pub trait Node: From<Expr<Self>> + AsRef<Expr<Self>> + AsMut<Expr<Self>> + Clone + Debug {}

impl<T> Node for T where T: From<Expr<T>> + AsRef<Expr<T>> + AsMut<Expr<T>> + Clone + Debug {}

pub trait FromPair: Sized {
    fn from_pair(_: Pair<'_>) -> Self;
}

pub trait IntoNode<T>: Sized {
    fn into_node(self) -> T;
}

impl<T: FromPair> IntoNode<T> for Pair<'_> {
    fn into_node(self) -> T {
        T::from_pair(self)
    }
}

impl FromPair for String {
    fn from_pair(p: Pair) -> Self {
        p.as_str().to_owned()
    }
}

impl<T: Node> FromPair for T {
    fn from_pair(p: Pair) -> T {
        match p.as_rule() {
            Rule::expr_type | Rule::paren_type | Rule::app_type | Rule::term_type => {
                p.into_inner().next().unwrap().into_node()
            }
            Rule::prim_type => Expr::Prim(p.into_node()).into(),
            Rule::rel_type => Expr::Rel(p.into_node()).into(),
            Rule::uri_type => Expr::Uri(p.into_node()).into(),
            Rule::array_type => Expr::Array(p.into_node()).into(),
            Rule::object_type => Expr::Object(p.into_node()).into(),
            Rule::var => Expr::Var(p.as_str().into()).into(),
            Rule::binding => Expr::Binding(p.as_str().into()).into(),
            Rule::join_type | Rule::any_type | Rule::sum_type => {
                let mut op = VariadicOp::from_pair(p);
                if op.exprs.len() == 1 {
                    op.exprs.remove(0)
                } else {
                    Expr::Op(op).into()
                }
            }
            Rule::apply => Expr::App(p.into_node()).into(),
            Rule::line => Expr::Ann(p.into_node()).into(),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program<T> {
    pub stmts: Vec<Statement<T>>,
}

impl<T: Node> FromPair for Program<T> {
    fn from_pair(p: Pair) -> Self {
        let stmts = p
            .into_inner()
            .flat_map(|p| match p.as_rule() {
                Rule::stmt => Some(p.into_node()),
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

impl<T: Node> FromPair for Declaration<T> {
    fn from_pair(p: Pair) -> Self {
        let mut p = p.into_inner();
        let name = p.nth(1).unwrap().as_str().into();
        let bindings: Vec<T> = p
            .next()
            .unwrap()
            .into_inner()
            .map(|p| p.into_node())
            .collect();
        let _hint = p.next().unwrap();
        let expr = p.next().unwrap().into_node();
        let expr = if bindings.is_empty() {
            expr
        } else {
            Expr::Lambda(Lambda {
                bindings,
                body: Box::new(expr),
            })
            .into()
        };
        Declaration { name, expr }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Resource<T> {
    pub rel: T,
}

impl<T: Node> FromPair for Resource<T> {
    fn from_pair(p: Pair) -> Self {
        Resource {
            rel: p.into_inner().nth(1).unwrap().into_node(),
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
pub struct Annotation<T> {
    pub ann: T,
}

impl<T: Node> FromPair for Annotation<T> {
    fn from_pair(p: Pair) -> Self {
        Annotation {
            ann: p.into_inner().next().unwrap().into_node(),
        }
    }
}

impl<'a, T> IntoIterator for &'a Annotation<T> {
    type Item = &'a T;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(&self.ann)
    }
}

impl<'a, T> IntoIterator for &'a mut Annotation<T> {
    type Item = &'a mut T;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(&mut self.ann)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<T> {
    Decl(Declaration<T>),
    Res(Resource<T>),
    Ann(Annotation<T>),
}

impl<T: Node> FromPair for Statement<T> {
    fn from_pair(p: Pair) -> Self {
        let p = p.into_inner().next().unwrap();
        match p.as_rule() {
            Rule::decl => Statement::Decl(p.into_node()),
            Rule::res => Statement::Res(p.into_node()),
            Rule::ann => Statement::Ann(p.into_node()),
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
    pub domain: Option<Box<T>>,
    pub range: Box<T>,
}

pub type Transfers<T> = EnumMap<Method, Option<Transfer<T>>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Relation<T> {
    pub uri: Box<T>,
    pub xfers: Transfers<T>,
}

impl<T: Node> FromPair for Relation<T> {
    fn from_pair(p: Pair) -> Self {
        let mut inner = p.into_inner();

        let uri: Box<_> = T::from_pair(inner.next().unwrap()).into();

        let mut xfers: Transfers<T> = EnumMap::default();

        for xfer in inner {
            let mut xfer = xfer.into_inner();

            let methods: Vec<_> = xfer
                .next()
                .unwrap()
                .into_inner()
                .map(|p| p.into_node())
                .collect();

            let domain = xfer
                .next()
                .unwrap()
                .into_inner()
                .next()
                .map(|p| Box::new(p.into_node()));

            let range: Box<_> = T::from_pair(xfer.next().unwrap()).into();

            for m in methods.iter() {
                xfers[*m] = Some(Transfer {
                    domain: domain.clone(),
                    range: range.clone(),
                })
            }
        }

        Relation { uri, xfers }
    }
}

impl<'a, T> IntoIterator for &'a Relation<T> {
    type Item = &'a T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        let it = once(self.uri.as_ref()).chain(
            self.xfers
                .values()
                .flatten()
                .flat_map(|xfer| {
                    if let Some(domain) = &xfer.domain {
                        [Some(xfer.range.as_ref()), Some(domain.as_ref())]
                    } else {
                        [Some(xfer.range.as_ref()), None]
                    }
                })
                .flatten(),
        );
        Box::new(it)
    }
}

impl<'a, T> IntoIterator for &'a mut Relation<T> {
    type Item = &'a mut T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        let it = once(self.uri.as_mut()).chain(
            self.xfers
                .values_mut()
                .flatten()
                .flat_map(|xfer| {
                    if let Some(domain) = &mut xfer.domain {
                        [Some(xfer.range.as_mut()), Some(domain.as_mut())]
                    } else {
                        [Some(xfer.range.as_mut()), None]
                    }
                })
                .flatten(),
        );
        Box::new(it)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UriSegment<T> {
    Literal(Literal),
    Variable(Property<T>),
}

impl<T> UriSegment<T> {
    pub fn root() -> Self {
        UriSegment::Literal("".into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Uri<T> {
    pub spec: Vec<UriSegment<T>>,
}

impl<'a, T> IntoIterator for &'a Uri<T> {
    type Item = &'a T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        let it = self.spec.iter().filter_map(|s| {
            if let UriSegment::Variable(t) = s {
                Some(&t.val)
            } else {
                None
            }
        });
        Box::new(it)
    }
}

impl<'a, T> IntoIterator for &'a mut Uri<T> {
    type Item = &'a mut T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        let it = self.spec.iter_mut().filter_map(|s| {
            if let UriSegment::Variable(t) = s {
                Some(&mut t.val)
            } else {
                None
            }
        });
        Box::new(it)
    }
}

impl<T: Node> FromPair for Uri<T> {
    fn from_pair(p: Pair) -> Self {
        let p = p.into_inner().next().unwrap();
        let spec: Vec<_> = match p.as_rule() {
            Rule::uri_kw => Default::default(),
            Rule::uri_root => vec![UriSegment::root()],
            Rule::uri_tpl => p
                .into_inner()
                .map(|p| match p.as_rule() {
                    Rule::uri_var => {
                        UriSegment::Variable(p.into_inner().next().unwrap().into_node())
                    }
                    Rule::uri_lit => UriSegment::Literal(p.as_str().into()),
                    _ => unreachable!(),
                })
                .collect(),
            _ => unreachable!(),
        };
        Uri { spec }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Array<T> {
    pub item: Box<T>,
}

impl<T: Node> FromPair for Array<T> {
    fn from_pair(p: Pair) -> Self {
        let item = Box::new(p.into_inner().next().unwrap().into_node());
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
    pub key: Ident,
    pub val: T,
}

impl<T: Node> FromPair for Property<T> {
    fn from_pair(p: Pair) -> Self {
        let mut inner = p.into_inner();
        let key = inner.next().unwrap().as_str().into();
        let val = inner.next().unwrap().into_node();
        Property { key, val }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Object<T> {
    pub props: Vec<Property<T>>,
}

impl<T> Default for Object<T> {
    fn default() -> Self {
        Object { props: Vec::new() }
    }
}

impl<T: Node> FromPair for Object<T> {
    fn from_pair(p: Pair) -> Self {
        let props = p.into_inner().map(|p| p.into_node()).collect();
        Object { props }
    }
}

impl<'a, T> IntoIterator for &'a Object<T> {
    type Item = &'a T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.props.iter().map(|p| &p.val))
    }
}

impl<'a, T> IntoIterator for &'a mut Object<T> {
    type Item = &'a mut T;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.props.iter_mut().map(|p| &mut p.val))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Operator {
    Join,
    Any,
    Sum,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariadicOp<T> {
    pub op: Operator,
    pub exprs: Vec<T>,
}

impl<T: Node> FromPair for VariadicOp<T> {
    fn from_pair(p: Pair) -> Self {
        let op = match p.as_rule() {
            Rule::join_type => Operator::Join,
            Rule::any_type => Operator::Any,
            Rule::sum_type => Operator::Sum,
            _ => unreachable!(),
        };
        let exprs = p.into_inner().map(|p| p.into_node()).collect();
        VariadicOp { op, exprs }
    }
}

impl<'a, T: Node> IntoIterator for &'a VariadicOp<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.exprs.iter()
    }
}

impl<'a, T: Node> IntoIterator for &'a mut VariadicOp<T> {
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

impl<T: Node> FromPair for Application<T> {
    fn from_pair(p: Pair) -> Self {
        let mut inner = p.into_inner();
        let name = inner.next().unwrap().as_str().into();
        let args = inner.into_iter().map(|p| p.into_node()).collect();
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
