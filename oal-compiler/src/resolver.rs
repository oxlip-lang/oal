use crate::environment::Env;
use crate::errors::Result;
use oal_syntax::ast::*;
use std::rc::Rc;

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

// TODO: a bare vector would be faster I guess.
type Path = Rc<List<Ident>>;

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

pub fn resolve(env: &Env, expr: &TypeExpr) -> Result<TypeExpr> {
    resolve_expr(env, Rc::new(List::Nil), expr)
}
