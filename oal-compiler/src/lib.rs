use std::collections::HashMap;
use std::rc::Rc;

use oal_syntax::ast::*;

type Env = HashMap<Ident, TypeExpr>;

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

fn well_type(expr: &TypeExpr) -> Result<TypeTag> {
    match expr {
        TypeExpr::Prim(_) => Ok(TypeTag::Prim),
        TypeExpr::Rel(rel) => {
            let uri = well_type(&rel.uri).and_then(|t| {
                if let TypeTag::Uri = t {
                    Ok(t)
                } else {
                    Err("expected uri as relation base".into())
                }
            });
            let range = well_type(&rel.range).and_then(|t| {
                if let TypeTag::Block = t {
                    Ok(t)
                } else {
                    Err("expected block as range".into())
                }
            });

            uri.and_then(|_| range.and_then(|_| Ok(TypeTag::Unknown)))
        }
        TypeExpr::Uri(uri) => {
            let r: Result<Vec<_>> = uri
                .spec
                .iter()
                .map(|s| match s {
                    UriSegment::Template(p) => well_type(&p.expr).and_then(|t| {
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
        TypeExpr::Sum(sum) => {
            let sum: Result<Vec<_>> = sum.iter().map(|e| well_type(e)).collect();

            sum.map(|sum| {
                sum.iter()
                    .reduce(|a, b| if a == b { a } else { &TypeTag::Unknown })
                    .unwrap_or(&TypeTag::Unknown)
                    .clone()
            })
        }
        TypeExpr::Var(_) => Err("unresolved variable".into()),
        TypeExpr::Join(join) => {
            let r: Result<Vec<_>> = join
                .iter()
                .map(|e| {
                    well_type(e).and_then(|t| {
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
        TypeExpr::Block(_) => Ok(TypeTag::Block),
    }
}

fn resolve_prop(env: &Env, from: Path, p: &Prop) -> Result<Prop> {
    resolve_expr(env, from, &p.expr).map(|e| Prop {
        ident: p.ident.clone(),
        expr: e,
    })
}

fn resolve_expr(env: &Env, from: Path, expr: &TypeExpr) -> Result<TypeExpr> {
    match expr {
        TypeExpr::Var(v) => {
            if from.contains(v) {
                Err("cycle detected".into())
            } else {
                let path = Rc::new(List::Cons(v.clone(), from));
                match env.get(v) {
                    None => Err("unknown identifier".into()),
                    Some(e) => resolve_expr(env, path, e),
                }
            }
        }
        TypeExpr::Prim(_) => Ok(expr.clone()),
        TypeExpr::Rel(rel) => {
            let uri = resolve_expr(env, from.clone(), &rel.uri);
            let methods = rel.methods.clone();
            let range = resolve_expr(env, from, &rel.range);

            uri.and_then(|uri| {
                range.and_then(|range| {
                    Ok(TypeExpr::Rel(TypeRel {
                        uri: Box::new(uri),
                        methods,
                        range: Box::new(range),
                    }))
                })
            })
        }
        TypeExpr::Uri(uri) => {
            let spec: Result<Vec<_>> = uri
                .spec
                .iter()
                .map(|s| match s {
                    UriSegment::Literal(_) => Ok(s.clone()),
                    UriSegment::Template(p) => {
                        resolve_prop(env, from.clone(), p).map(|p| UriSegment::Template(p))
                    }
                })
                .collect();

            spec.map(|spec| TypeExpr::Uri(TypeUri { spec }))
        }
        TypeExpr::Block(block) => {
            let props: Result<Vec<_>> = block
                .iter()
                .map(|p| resolve_prop(env, from.clone(), p))
                .collect();

            props.map(|props| TypeExpr::Block(TypeBlock(props)))
        }
        TypeExpr::Sum(sum) => {
            let sum: Result<Vec<_>> = sum
                .iter()
                .map(|e| resolve_expr(env, from.clone(), e))
                .collect();

            sum.map(|sum| TypeExpr::Sum(TypeSum(sum)))
        }
        TypeExpr::Join(join) => {
            let join: Result<Vec<_>> = join
                .iter()
                .map(|e| resolve_expr(env, from.clone(), e))
                .collect();

            join.map(|join| TypeExpr::Join(TypeJoin(join)))
        }
    }
}

fn environment(d: &Doc) -> Env {
    d.stmts
        .iter()
        .flat_map(|s| match s {
            Stmt::Decl(d) => Some((d.var.clone(), d.expr.clone())),
            _ => None,
        })
        .collect()
}

pub fn visit(d: &Doc) {
    let env = environment(&d);

    let resources: Vec<_> = d
        .stmts
        .iter()
        .flat_map(|s| match s {
            Stmt::Res(r) => Some(&r.rel),
            _ => None,
        })
        .map(|e| {
            resolve_expr(&env, Rc::new(List::Nil), e).and_then(|e| well_type(&e).map(|t| (e, t)))
        })
        .collect();

    println!("{:#?}", resources)
}
