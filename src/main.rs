extern crate pest;
#[macro_use]
extern crate pest_derive;

use indexmap::indexmap;
use openapiv3::{Info, OpenAPI};
use pest::Parser;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct MyParser;

type Pair<'a> = pest::iterators::Pair<'a, Rule>;

#[derive(Clone, Debug)]
enum TypeAny {
    Prim(TypePrim),
    Rel(TypeRel),
    Uri(TypeUri),
    Join(TypeJoin),
    Block(TypeBlock),
    Sum(TypeSum),
    Var(Ident),
}

#[derive(Clone, Debug)]
struct Doc {
    stmts: Vec<Stmt>,
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

#[derive(Clone, Debug)]
enum Stmt {
    Decl { var: Ident, expr: TypeAny },
    Res { rel: TypeAny },
}

impl From<Pair<'_>> for Stmt {
    fn from(p: Pair) -> Self {
        let p = p.into_inner().next().unwrap();
        match p.as_rule() {
            Rule::decl => {
                let mut p = p.into_inner();
                let var = p.nth(1).unwrap().as_str().into();
                let expr = p.next().unwrap().into();
                Stmt::Decl { var, expr }
            }
            Rule::res => Stmt::Res {
                rel: p.into_inner().nth(1).unwrap().into(),
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
enum Method {
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

#[derive(Clone, Debug)]
struct TypeRel {
    uri: Box<TypeAny>,
    methods: Vec<Method>,
    range: Box<TypeAny>,
}

impl From<Pair<'_>> for TypeRel {
    fn from(p: Pair) -> Self {
        let mut inner = p.into_inner();

        let uri: Box<_> = TypeAny::from(inner.next().unwrap()).into();

        let methods: Vec<_> = inner
            .next()
            .unwrap()
            .into_inner()
            .map(|p| p.into())
            .collect();

        let range: Box<_> = TypeAny::from(inner.next().unwrap()).into();

        TypeRel {
            uri,
            methods,
            range,
        }
    }
}

#[derive(Clone, Debug)]
enum UriSegment {
    Literal(Rc<str>),
    Template(Prop),
}

#[derive(Clone, Debug)]
struct TypeUri {
    spec: Vec<UriSegment>,
}

impl From<Pair<'_>> for TypeUri {
    fn from(p: Pair) -> Self {
        let mut p = p.into_inner();
        p.next();
        let spec: Vec<_> = p
            .next()
            .map(|p| {
                p.into_inner()
                    .map(|p| match p.as_rule() {
                        Rule::uri_tpl => {
                            UriSegment::Template(p.into_inner().next().unwrap().into())
                        }
                        Rule::uri_lit => {
                            UriSegment::Literal(p.into_inner().next().unwrap().as_str().into())
                        }
                        _ => unreachable!(),
                    })
                    .collect()
            })
            .unwrap_or(vec![]);
        TypeUri { spec }
    }
}

type Ident = Rc<str>;

#[derive(Clone, Debug)]
struct Prop(Ident, TypeAny);

impl From<Pair<'_>> for Prop {
    fn from(p: Pair) -> Self {
        let mut inner = p.into_inner();
        let id = inner.next().unwrap().as_str().into();
        let expr = inner.next().unwrap().into();
        Prop(id, expr)
    }
}

#[derive(Clone, Debug)]
struct TypeBlock(Vec<Prop>);

impl From<Pair<'_>> for TypeBlock {
    fn from(p: Pair) -> Self {
        TypeBlock(p.into_inner().map(|p| p.into()).collect())
    }
}

#[derive(Clone, Debug)]
struct TypeJoin(Vec<TypeAny>);

impl From<Pair<'_>> for TypeJoin {
    fn from(p: Pair) -> Self {
        TypeJoin(p.into_inner().map(|p| p.into()).collect())
    }
}

#[derive(Clone, Debug)]
struct TypeSum(Vec<TypeAny>);

impl From<Pair<'_>> for TypeSum {
    fn from(p: Pair) -> Self {
        let types = p
            .into_inner()
            .map(|p| p.into_inner().next().unwrap().into())
            .collect();
        TypeSum(types)
    }
}

#[derive(Clone, Debug)]
enum TypePrim {
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

impl From<Pair<'_>> for TypeAny {
    fn from(p: Pair<'_>) -> Self {
        match p.as_rule() {
            Rule::prim_type => TypeAny::Prim(p.into()),
            Rule::rel_type => TypeAny::Rel(p.into()),
            Rule::uri_type => TypeAny::Uri(p.into()),
            Rule::join_type => match TypeJoin::from(p) {
                TypeJoin(join) if join.len() == 1 => join.first().unwrap().clone(),
                t @ _ => TypeAny::Join(t),
            },
            Rule::sum_type => match TypeSum::from(p) {
                TypeSum(sum) if sum.len() == 1 => sum.first().unwrap().clone(),
                t @ _ => TypeAny::Sum(t),
            },
            Rule::block_type => TypeAny::Block(p.into()),
            Rule::ident => TypeAny::Var(p.as_str().into()),
            _ => unreachable!(),
        }
    }
}

type Env = HashMap<Ident, TypeAny>;

#[derive(Debug)]
enum List<T> {
    Nil,
    Cons(T, Rc<List<T>>),
}

impl<T: Eq> List<T> {
    fn contains(&self, x: &T) -> bool {
        match self {
            Self::Nil => false,
            Self::Cons(h, t) => x == h || t.contains(x),
        }
    }
}

type Path = Rc<List<Ident>>;

#[derive(Debug, Clone)]
struct EvalError {
    msg: String,
}

impl EvalError {
    fn new(msg: &str) -> EvalError {
        EvalError { msg: msg.into() }
    }
}

impl From<&str> for EvalError {
    fn from(msg: &str) -> Self {
        Self::new(msg)
    }
}

type Result<T> = std::result::Result<T, EvalError>;

#[derive(PartialEq, Clone, Copy, Debug)]
enum TypeTag {
    Prim,
    Uri,
    Block,
    Unknown,
}

fn welltype(expr: &TypeAny) -> Result<TypeTag> {
    match expr {
        TypeAny::Prim(_) => Ok(TypeTag::Prim),
        TypeAny::Rel(TypeRel {
            uri,
            methods: _methods,
            range,
        }) => {
            let uri = welltype(uri).and_then(|t| {
                if let TypeTag::Uri = t {
                    Ok(t)
                } else {
                    Err("expected uri as relation base".into())
                }
            });
            let range = welltype(range).and_then(|t| {
                if let TypeTag::Block = t {
                    Ok(t)
                } else {
                    Err("expected block as range".into())
                }
            });

            uri.and_then(|_| range.and_then(|_| Ok(TypeTag::Unknown)))
        }
        TypeAny::Uri(TypeUri { spec }) => {
            let r: Result<Vec<_>> = spec
                .iter()
                .map(|s| match s {
                    UriSegment::Template(Prop(_, e)) => welltype(e).and_then(|t| {
                        if let TypeTag::Prim = t {
                            Ok(())
                        } else {
                            Err("expected prim as uri template property".into())
                        }
                    }),
                    UriSegment::Literal(_) => Ok(()),
                })
                .collect();

            r.map(|_| TypeTag::Uri)
        }
        TypeAny::Sum(TypeSum(sum)) => {
            let sum: Result<Vec<_>> = sum.iter().map(|e| welltype(e)).collect();

            sum.map(|sum| {
                sum.iter()
                    .reduce(|a, b| if a == b { a } else { &TypeTag::Unknown })
                    .unwrap_or(&TypeTag::Unknown)
                    .clone()
            })
        }
        TypeAny::Var(_) => Err("unresolved variable".into()),
        TypeAny::Join(TypeJoin(join)) => {
            let r: Result<Vec<_>> = join
                .iter()
                .map(|e| {
                    welltype(e).and_then(|t| {
                        if t == TypeTag::Block {
                            Ok(())
                        } else {
                            Err("expected block as join element".into())
                        }
                    })
                })
                .collect();

            r.map(|_| TypeTag::Block)
        }
        TypeAny::Block(_) => Ok(TypeTag::Block),
    }
}

fn resolve(env: &Env, from: Path, expr: &TypeAny) -> Result<TypeAny> {
    match expr {
        TypeAny::Var(v) => {
            if from.contains(v) {
                Err("cycle detected".into())
            } else {
                let path = Rc::new(List::Cons(v.clone(), from));
                match env.get(v) {
                    None => Err("unknown identifier".into()),
                    Some(e) => resolve(env, path, e),
                }
            }
        }
        TypeAny::Prim(_) => Ok(expr.clone()),
        TypeAny::Rel(TypeRel {
            uri,
            methods,
            range,
        }) => {
            let uri = resolve(env, from.clone(), uri);
            let methods = methods.clone();
            let range = resolve(env, from, range);

            uri.and_then(|uri| {
                range.and_then(|range| {
                    Ok(TypeAny::Rel(TypeRel {
                        uri: Box::new(uri),
                        methods,
                        range: Box::new(range),
                    }))
                })
            })
        }
        TypeAny::Uri(TypeUri { spec }) => {
            let spec: Result<Vec<_>> = spec
                .iter()
                .map(|s| match s {
                    UriSegment::Literal(_) => Ok(s.clone()),
                    UriSegment::Template(Prop(id, e)) => resolve(env, from.clone(), e)
                        .map(|e| UriSegment::Template(Prop(id.clone(), e))),
                })
                .collect();

            spec.map(|spec| TypeAny::Uri(TypeUri { spec }))
        }
        TypeAny::Block(TypeBlock(props)) => {
            let props: Result<Vec<_>> = props
                .iter()
                .map(|Prop(p, e)| resolve(env, from.clone(), e).map(|e| Prop(p.clone(), e)))
                .collect();

            props.map(|props| TypeAny::Block(TypeBlock(props)))
        }
        TypeAny::Sum(TypeSum(sum)) => {
            let sum: Result<Vec<_>> = sum.iter().map(|e| resolve(env, from.clone(), e)).collect();

            sum.map(|sum| TypeAny::Sum(TypeSum(sum)))
        }
        TypeAny::Join(TypeJoin(join)) => {
            let join: Result<Vec<_>> = join.iter().map(|e| resolve(env, from.clone(), e)).collect();

            join.map(|join| TypeAny::Join(TypeJoin(join)))
        }
    }
}

fn environment(d: &Doc) -> Env {
    d.stmts
        .iter()
        .flat_map(|s| match s {
            Stmt::Decl { var, expr } => Some((var.clone(), expr.clone())),
            _ => None,
        })
        .collect()
}

fn visit(d: &Doc) {
    let env = environment(&d);

    let resources: Vec<_> = d
        .stmts
        .iter()
        .flat_map(|s| match s {
            Stmt::Res { rel } => Some(rel),
            _ => None,
        })
        .map(|e| resolve(&env, Rc::new(List::Nil), e).and_then(|e| welltype(&e).map(|t| (e, t))))
        .collect();

    println!("{:#?}", resources)
}

fn main() {
    let unparsed_file = fs::read_to_string("src/doc.txt").expect("cannot read file");

    let p: Pair = MyParser::parse(Rule::doc, &unparsed_file)
        .expect("parsing failed")
        .next()
        .unwrap();

    let doc = Doc::from(p);

    println!("{:#?}", doc);

    visit(&doc);

    let spec = OpenAPI {
        openapi: "3.0.1".into(),
        info: Info {
            title: "Test OpenAPI specification".into(),
            description: None,
            terms_of_service: None,
            contact: None,
            license: None,
            version: "0.1.0".into(),
            extensions: indexmap! {},
        },
        servers: vec![],
        paths: Default::default(),
        components: None,
        security: None,
        tags: vec![],
        external_docs: None,
        extensions: indexmap! {},
    };

    let output = serde_yaml::to_string(&spec).unwrap();

    println!("{}", output);
}
