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

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Prim(Prim),
    Rel(Rel),
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
pub struct Typed<T> {
    pub tag: Option<Tag>,
    pub inner: T,
}

impl<T> Typed<T> {
    pub fn unwrap_tag(&self) -> Tag {
        self.tag.as_ref().unwrap().clone()
    }
}

impl<T> From<T> for Typed<T> {
    fn from(e: T) -> Self {
        Typed {
            tag: None,
            inner: e,
        }
    }
}

pub type TypedIdent = Typed<Ident>;
pub type TypedExpr = Typed<Expr>;

impl From<Pair<'_>> for TypedExpr {
    fn from(p: Pair<'_>) -> Self {
        match p.as_rule() {
            Rule::expr_type | Rule::paren_type | Rule::app_type | Rule::term_type => {
                p.into_inner().next().unwrap().into()
            }
            Rule::prim_type => Expr::Prim(p.into()).into(),
            Rule::rel_type => Expr::Rel(p.into()).into(),
            Rule::uri_type => Expr::Uri(p.into()).into(),
            Rule::array_type => Expr::Array(p.into()).into(),
            Rule::object_type => Expr::Object(p.into()).into(),
            Rule::var => Expr::Var(p.as_str().into()).into(),
            Rule::binding => Expr::Binding(p.as_str().into()).into(),
            Rule::join_type | Rule::any_type | Rule::sum_type => {
                let op = VariadicOp::from(p);
                if op.exprs.len() == 1 {
                    op.exprs.first().unwrap().clone()
                } else {
                    Expr::Op(op).into()
                }
            }
            Rule::apply => Expr::App(p.into()).into(),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

impl From<Pair<'_>> for Program {
    fn from(p: Pair) -> Self {
        let stmts = p
            .into_inner()
            .flat_map(|p| match p.as_rule() {
                Rule::stmt => Some(p.into()),
                _ => None,
            })
            .collect();
        Program { stmts }
    }
}

impl<'a> IntoIterator for &'a Program {
    type Item = &'a Stmt;
    type IntoIter = Iter<'a, Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.stmts.iter()
    }
}

impl<'a> IntoIterator for &'a mut Program {
    type Item = &'a mut Stmt;
    type IntoIter = IterMut<'a, Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.stmts.iter_mut()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Decl {
    pub name: Ident,
    pub expr: TypedExpr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Res {
    pub rel: TypedExpr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Annotation {
    pub ann: String,
}

impl From<Pair<'_>> for Annotation {
    fn from(p: Pair) -> Self {
        Annotation {
            ann: p.as_str().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Decl(Decl),
    Res(Res),
    Ann(Annotation),
}

impl From<Pair<'_>> for Stmt {
    fn from(p: Pair) -> Self {
        let p = p.into_inner().next().unwrap();
        match p.as_rule() {
            Rule::decl => {
                let mut p = p.into_inner();
                let name = p.nth(1).unwrap().as_str().into();
                let bindings: Vec<TypedExpr> =
                    p.next().unwrap().into_inner().map(|p| p.into()).collect();
                let _hint = p.next().unwrap();
                let expr = p.next().unwrap().into();
                let expr = if bindings.is_empty() {
                    expr
                } else {
                    Expr::Lambda(Lambda {
                        bindings,
                        body: Box::new(expr),
                    })
                    .into()
                };
                Stmt::Decl(Decl { name, expr })
            }
            Rule::res => Stmt::Res(Res {
                rel: p.into_inner().nth(1).unwrap().into(),
            }),
            Rule::ann => Stmt::Ann(p.into_inner().next().unwrap().into()),
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

impl From<Pair<'_>> for Method {
    fn from(p: Pair) -> Self {
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
pub struct Rel {
    pub uri: Box<TypedExpr>,
    pub xfers: Transfers,
}

impl From<Pair<'_>> for Rel {
    fn from(p: Pair) -> Self {
        let mut inner = p.into_inner();

        let uri: Box<_> = TypedExpr::from(inner.next().unwrap()).into();

        let mut xfers: Transfers = EnumMap::default();

        for xfer in inner {
            let mut xfer = xfer.into_inner();

            let methods: Vec<_> = xfer
                .next()
                .unwrap()
                .into_inner()
                .map(|p| p.into())
                .collect();

            let domain = xfer
                .next()
                .unwrap()
                .into_inner()
                .next()
                .map(|p| Box::new(p.into()));

            let range: Box<_> = TypedExpr::from(xfer.next().unwrap()).into();

            for m in methods.iter() {
                xfers[*m] = Some(Transfer {
                    domain: domain.clone(),
                    range: range.clone(),
                })
            }
        }

        Rel { uri, xfers }
    }
}

impl<'a> IntoIterator for &'a Rel {
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

impl<'a> IntoIterator for &'a mut Rel {
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
    Variable(Prop),
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

impl From<Pair<'_>> for Uri {
    fn from(p: Pair) -> Self {
        let p = p.into_inner().next().unwrap();
        let spec: Vec<_> = match p.as_rule() {
            Rule::uri_kw => Default::default(),
            Rule::uri_root => vec![UriSegment::root()],
            Rule::uri_tpl => p
                .into_inner()
                .map(|p| match p.as_rule() {
                    Rule::uri_var => UriSegment::Variable(p.into_inner().next().unwrap().into()),
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

impl From<Pair<'_>> for Array {
    fn from(p: Pair) -> Self {
        let item = Box::new(p.into_inner().next().unwrap().into());
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
pub struct Prop {
    pub key: Ident,
    pub val: TypedExpr,
}

impl From<Pair<'_>> for Prop {
    fn from(p: Pair) -> Self {
        let mut inner = p.into_inner();
        let key = inner.next().unwrap().as_str().into();
        let val = inner.next().unwrap().into();
        Prop { key, val }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Object {
    pub props: Vec<Prop>,
}

impl From<Pair<'_>> for Object {
    fn from(p: Pair) -> Self {
        let props = p.into_inner().map(|p| p.into()).collect();
        Object { props }
    }
}

impl Object {
    pub fn iter(&self) -> Iter<Prop> {
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

impl From<Pair<'_>> for VariadicOp {
    fn from(p: Pair) -> Self {
        let op = match p.as_rule() {
            Rule::join_type => Operator::Join,
            Rule::any_type => Operator::Any,
            Rule::sum_type => Operator::Sum,
            _ => unreachable!(),
        };
        let exprs = p.into_inner().map(|p| p.into()).collect();
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
pub enum Prim {
    Num,
    Str,
    Bool,
}

impl From<Pair<'_>> for Prim {
    fn from(p: Pair) -> Self {
        match p.into_inner().next().unwrap().as_rule() {
            Rule::num_kw => Prim::Num,
            Rule::str_kw => Prim::Str,
            Rule::bool_kw => Prim::Bool,
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

impl From<Pair<'_>> for Application {
    fn from(p: Pair) -> Self {
        let mut inner = p.into_inner();
        let name = inner.next().unwrap().as_str().into();
        let args = inner.into_iter().map(|p| p.into()).collect();
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
