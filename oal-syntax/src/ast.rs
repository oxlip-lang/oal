use crate::{Pair, Rule};
use std::rc::Rc;

pub type Literal = Rc<str>;
pub type Ident = Rc<str>;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TypeTag {
    Number,
    String,
    Boolean,
    Relation,
    Object,
    Uri,
    Any,
    Var(usize),
}

impl TypeTag {
    pub fn is_primitive(&self) -> bool {
        *self == Self::Number || *self == Self::String || *self == Self::Boolean
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeExpr {
    Prim(TypePrim),
    Rel(TypeRel),
    Uri(TypeUri),
    Join(TypeJoin),
    Block(TypeBlock),
    Sum(TypeSum),
    Var(Ident),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Doc {
    pub stmts: Vec<Stmt>,
}

impl From<Pair<'_>> for Doc {
    fn from(p: Pair) -> Self {
        let stmts = p
            .into_inner()
            .flat_map(|p| match p.as_rule() {
                Rule::stmt => Some(p.into()),
                _ => None,
            })
            .collect();
        Doc { stmts }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StmtDecl {
    pub var: Ident,
    pub tag: TypeTag,
    pub expr: TypeExpr,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StmtRes {
    pub rel: TypeExpr,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Decl(StmtDecl),
    Res(StmtRes),
}

impl From<Pair<'_>> for Stmt {
    fn from(p: Pair) -> Self {
        let p = p.into_inner().next().unwrap();
        match p.as_rule() {
            Rule::decl => {
                let mut p = p.into_inner();
                let var_pair = p.nth(1).unwrap();
                let tag = TypeTag::Var(var_pair.as_span().start());
                let var = var_pair.as_str().into();
                let next_pair = p.next().unwrap();
                let expr = if next_pair.as_rule() == Rule::type_kw {
                    // TODO: parse the type annotation into a type tag
                    let _ann = next_pair;
                    p.next().unwrap()
                } else {
                    next_pair
                };
                let expr = expr.into();
                Stmt::Decl(StmtDecl { var, tag, expr })
            }
            Rule::res => Stmt::Res(StmtRes {
                rel: p.into_inner().nth(1).unwrap().into(),
            }),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Method {
    Get,
    Put,
}

impl From<Pair<'_>> for Method {
    fn from(p: Pair) -> Self {
        match p.into_inner().next().unwrap().as_rule() {
            Rule::get_kw => Method::Get,
            Rule::put_kw => Method::Put,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeRel {
    pub uri: Box<TypeExpr>,
    pub methods: Vec<Method>,
    pub range: Box<TypeExpr>,
}

impl From<Pair<'_>> for TypeRel {
    fn from(p: Pair) -> Self {
        let mut inner = p.into_inner();

        let uri: Box<_> = TypeExpr::from(inner.next().unwrap()).into();

        let methods: Vec<_> = inner
            .next()
            .unwrap()
            .into_inner()
            .map(|p| p.into())
            .collect();

        let range: Box<_> = TypeExpr::from(inner.next().unwrap()).into();

        TypeRel {
            uri,
            methods,
            range,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum UriSegment {
    Literal(Literal),
    Template(Prop),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeUri {
    pub spec: Vec<UriSegment>,
}

impl TypeUri {
    pub fn is_empty(&self) -> bool {
        self.spec.is_empty()
    }
}

impl<'a> IntoIterator for &'a TypeUri {
    type Item = &'a UriSegment;
    type IntoIter = core::slice::Iter<'a, UriSegment>;

    fn into_iter(self) -> Self::IntoIter {
        self.spec.iter()
    }
}

impl From<Pair<'_>> for TypeUri {
    fn from(p: Pair) -> Self {
        let p = p.into_inner().next().unwrap();
        let spec: Vec<_> = match p.as_rule() {
            Rule::uri_spec => p
                .into_inner()
                .map(|p| match p.as_rule() {
                    Rule::uri_tpl => UriSegment::Template(p.into_inner().next().unwrap().into()),
                    Rule::uri_lit => {
                        UriSegment::Literal(p.into_inner().next().unwrap().as_str().into())
                    }
                    _ => unreachable!(),
                })
                .collect(),
            _ => Default::default(),
        };
        TypeUri { spec }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prop {
    pub ident: Ident,
    pub tag: TypeTag,
    pub expr: TypeExpr,
}

impl From<Pair<'_>> for Prop {
    fn from(p: Pair) -> Self {
        let mut inner = p.into_inner();
        let ident_pair = inner.next().unwrap();
        let ident = ident_pair.as_str().into();
        let tag = TypeTag::Var(ident_pair.as_span().start());
        let expr = inner.next().unwrap().into();
        Prop { ident, tag, expr }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeBlock {
    pub props: Vec<Prop>,
}

impl From<Pair<'_>> for TypeBlock {
    fn from(p: Pair) -> Self {
        let props = p.into_inner().map(|p| p.into()).collect();
        TypeBlock { props }
    }
}

impl TypeBlock {
    pub fn iter(&self) -> std::slice::Iter<Prop> {
        self.props.iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeJoin {
    pub exprs: Vec<TypeExpr>,
}

impl From<Pair<'_>> for TypeJoin {
    fn from(p: Pair) -> Self {
        let exprs = p
            .into_inner()
            .map(|p| p.into_inner().next().unwrap().into())
            .collect();
        TypeJoin { exprs }
    }
}

impl TypeJoin {
    pub fn iter(&self) -> std::slice::Iter<TypeExpr> {
        self.exprs.iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeSum {
    pub exprs: Vec<TypeExpr>,
}

impl From<Pair<'_>> for TypeSum {
    fn from(p: Pair) -> Self {
        let exprs = p
            .into_inner()
            .map(|p| p.into_inner().next().unwrap().into())
            .collect();
        TypeSum { exprs }
    }
}

impl TypeSum {
    pub fn iter(&self) -> std::slice::Iter<TypeExpr> {
        self.exprs.iter()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypePrim {
    Num,
    Str,
    Bool,
}

impl From<Pair<'_>> for TypePrim {
    fn from(p: Pair) -> Self {
        match p.into_inner().next().unwrap().as_rule() {
            Rule::num_kw => TypePrim::Num,
            Rule::str_kw => TypePrim::Str,
            Rule::bool_kw => TypePrim::Bool,
            _ => unreachable!(),
        }
    }
}

impl From<Pair<'_>> for TypeExpr {
    fn from(p: Pair<'_>) -> Self {
        match p.as_rule() {
            Rule::prim_type => TypeExpr::Prim(p.into()),
            Rule::rel_type => TypeExpr::Rel(p.into()),
            Rule::uri_type => TypeExpr::Uri(p.into()),
            Rule::join_type => {
                let join = TypeJoin::from(p);
                if join.exprs.len() == 1 {
                    join.exprs.first().unwrap().clone()
                } else {
                    TypeExpr::Join(join)
                }
            }
            Rule::sum_type => {
                let sum = TypeSum::from(p);
                if sum.exprs.len() == 1 {
                    sum.exprs.first().unwrap().clone()
                } else {
                    TypeExpr::Sum(sum)
                }
            }
            Rule::block_type => TypeExpr::Block(p.into()),
            Rule::paren_type => p.into_inner().next().unwrap().into(),
            Rule::ident => TypeExpr::Var(p.as_str().into()),
            _ => unreachable!(),
        }
    }
}
