use crate::{Pair, Rule};
use enum_map::{Enum, EnumMap};
use std::iter::{once, Once};
use std::rc::Rc;
use std::slice::{Iter, IterMut};

pub type Literal = Rc<str>;
pub type Ident = Rc<str>;

#[derive(PartialEq, Clone, Debug)]
pub struct FuncTag {
    pub bindings: Vec<Tag>,
    pub range: Box<Tag>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Tag {
    Primitive,
    Relation,
    Object,
    Array,
    Uri,
    Any,
    Func(FuncTag),
    Var(usize),
}

impl Tag {
    pub fn is_variable(&self) -> bool {
        if let Tag::Var(_) = self {
            true
        } else {
            false
        }
    }
}

pub trait Tagged {
    fn tag(&self) -> Option<&Tag>;
    fn set_tag(&mut self, t: Tag);
    fn unwrap_tag(&self) -> Tag;
    fn with_tag(self, t: Tag) -> Self;
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Prim(Primitive),
    Rel(Relation),
    Uri(Uri),
    Array(Array),
    Object(Object),
    Op(VariadicOp),
    Var(Ident),
    Lambda(Lambda),
    App(Application),
    Binding(Ident),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedExpr {
    tag: Option<Tag>,
    inner: Expr,
}

impl Tagged for TypedExpr {
    fn tag(&self) -> Option<&Tag> {
        self.tag.as_ref()
    }

    fn set_tag(&mut self, t: Tag) {
        self.tag = Some(t)
    }

    fn unwrap_tag(&self) -> Tag {
        self.tag.as_ref().unwrap().clone()
    }

    fn with_tag(mut self, t: Tag) -> Self {
        self.set_tag(t);
        self
    }
}

impl From<Expr> for TypedExpr {
    fn from(e: Expr) -> Self {
        TypedExpr {
            tag: None,
            inner: e,
        }
    }
}

impl AsRef<Expr> for TypedExpr {
    fn as_ref(&self) -> &Expr {
        &self.inner
    }
}

impl AsMut<Expr> for TypedExpr {
    fn as_mut(&mut self) -> &mut Expr {
        &mut self.inner
    }
}

pub trait FromPair: Sized {
    fn from_pair(_: Pair<'_>) -> Self;
}

pub trait IntoNode<T>: Sized {
    fn into_node(self) -> T;
}

impl<T> IntoNode<T> for Pair<'_>
where
    T: FromPair,
{
    fn into_node(self) -> T {
        T::from_pair(self)
    }
}

impl FromPair for TypedExpr {
    fn from_pair(p: Pair) -> Self {
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
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub stmts: Vec<Statement>,
}

impl FromPair for Program {
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

impl<'a> IntoIterator for &'a Program {
    type Item = &'a Statement;
    type IntoIter = Iter<'a, Statement>;

    fn into_iter(self) -> Self::IntoIter {
        self.stmts.iter()
    }
}

impl<'a> IntoIterator for &'a mut Program {
    type Item = &'a mut Statement;
    type IntoIter = IterMut<'a, Statement>;

    fn into_iter(self) -> Self::IntoIter {
        self.stmts.iter_mut()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    pub name: Ident,
    pub expr: TypedExpr,
}

impl FromPair for Declaration {
    fn from_pair(p: Pair) -> Self {
        let mut p = p.into_inner();
        let name = p.nth(1).unwrap().as_str().into();
        let bindings: Vec<TypedExpr> = p
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
pub struct Resource {
    pub rel: TypedExpr,
}

impl FromPair for Resource {
    fn from_pair(p: Pair) -> Self {
        Resource {
            rel: p.into_inner().nth(1).unwrap().into_node(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Annotation {
    pub ann: String,
}

impl FromPair for Annotation {
    fn from_pair(p: Pair) -> Self {
        Annotation {
            ann: p.into_inner().next().unwrap().as_str().to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Decl(Declaration),
    Res(Resource),
    Ann(Annotation),
}

impl FromPair for Statement {
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
pub struct Transfer {
    pub domain: Option<Box<TypedExpr>>,
    pub range: Box<TypedExpr>,
}

pub type Transfers = EnumMap<Method, Option<Transfer>>;

#[derive(Clone, Debug, PartialEq)]
pub struct Relation {
    pub uri: Box<TypedExpr>,
    pub xfers: Transfers,
}

impl FromPair for Relation {
    fn from_pair(p: Pair) -> Self {
        let mut inner = p.into_inner();

        let uri: Box<_> = TypedExpr::from_pair(inner.next().unwrap()).into();

        let mut xfers: Transfers = EnumMap::default();

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

            let range: Box<_> = TypedExpr::from_pair(xfer.next().unwrap()).into();

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

impl<'a> IntoIterator for &'a Relation {
    type Item = &'a TypedExpr;
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

impl<'a> IntoIterator for &'a mut Relation {
    type Item = &'a mut TypedExpr;
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
pub enum UriSegment {
    Literal(Literal),
    Variable(Property),
}

impl UriSegment {
    pub fn root() -> Self {
        UriSegment::Literal("".into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Uri {
    pub spec: Vec<UriSegment>,
}

impl<'a> IntoIterator for &'a Uri {
    type Item = &'a TypedExpr;
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

impl<'a> IntoIterator for &'a mut Uri {
    type Item = &'a mut TypedExpr;
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

impl FromPair for Uri {
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
pub struct Array {
    pub item: Box<TypedExpr>,
}

impl FromPair for Array {
    fn from_pair(p: Pair) -> Self {
        let item = Box::new(p.into_inner().next().unwrap().into_node());
        Array { item }
    }
}

impl<'a> IntoIterator for &'a Array {
    type Item = &'a TypedExpr;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(self.item.as_ref())
    }
}

impl<'a> IntoIterator for &'a mut Array {
    type Item = &'a mut TypedExpr;
    type IntoIter = Once<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        once(self.item.as_mut())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Property {
    pub key: Ident,
    pub val: TypedExpr,
}

impl FromPair for Property {
    fn from_pair(p: Pair) -> Self {
        let mut inner = p.into_inner();
        let key = inner.next().unwrap().as_str().into();
        let val = inner.next().unwrap().into_node();
        Property { key, val }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Object {
    pub props: Vec<Property>,
}

impl FromPair for Object {
    fn from_pair(p: Pair) -> Self {
        let props = p.into_inner().map(|p| p.into_node()).collect();
        Object { props }
    }
}

impl Object {
    pub fn iter(&self) -> Iter<Property> {
        self.props.iter()
    }
}

impl<'a> IntoIterator for &'a Object {
    type Item = &'a TypedExpr;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.props.iter().map(|p| &p.val))
    }
}

impl<'a> IntoIterator for &'a mut Object {
    type Item = &'a mut TypedExpr;
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
pub struct VariadicOp {
    pub op: Operator,
    pub exprs: Vec<TypedExpr>,
}

impl FromPair for VariadicOp {
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

impl VariadicOp {
    pub fn iter(&self) -> std::slice::Iter<TypedExpr> {
        self.exprs.iter()
    }
}

impl<'a> IntoIterator for &'a VariadicOp {
    type Item = &'a TypedExpr;
    type IntoIter = Iter<'a, TypedExpr>;

    fn into_iter(self) -> Self::IntoIter {
        self.exprs.iter()
    }
}

impl<'a> IntoIterator for &'a mut VariadicOp {
    type Item = &'a mut TypedExpr;
    type IntoIter = IterMut<'a, TypedExpr>;

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
pub struct Lambda {
    pub bindings: Vec<TypedExpr>,
    pub body: Box<TypedExpr>,
}

impl<'a> IntoIterator for &'a Lambda {
    type Item = &'a TypedExpr;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.bindings.iter().chain(once(self.body.as_ref())))
    }
}

impl<'a> IntoIterator for &'a mut Lambda {
    type Item = &'a mut TypedExpr;
    type IntoIter = Box<dyn Iterator<Item = Self::Item> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.bindings.iter_mut().chain(once(self.body.as_mut())))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Application {
    pub name: Ident,
    pub args: Vec<TypedExpr>,
}

impl FromPair for Application {
    fn from_pair(p: Pair) -> Self {
        let mut inner = p.into_inner();
        let name = inner.next().unwrap().as_str().into();
        let args = inner.into_iter().map(|p| p.into_node()).collect();
        Application { name, args }
    }
}

impl<'a> IntoIterator for &'a Application {
    type Item = &'a TypedExpr;
    type IntoIter = Iter<'a, TypedExpr>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.iter()
    }
}

impl<'a> IntoIterator for &'a mut Application {
    type Item = &'a mut TypedExpr;
    type IntoIter = IterMut<'a, TypedExpr>;

    fn into_iter(self) -> Self::IntoIter {
        self.args.iter_mut()
    }
}
