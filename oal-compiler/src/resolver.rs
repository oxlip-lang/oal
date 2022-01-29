use crate::errors::Result;
use crate::scope::Scope;
use oal_syntax::ast::*;

type Path = Vec<Ident>;

fn resolve_prop(env: &Scope, from: Path, p: &Prop) -> Result<Prop> {
    resolve_expr(env, from, &p.val).map(|e| Prop {
        key: p.key.clone(),
        val: e,
    })
}

fn resolve_expr(env: &Scope, from: Path, expr: &TypedExpr) -> Result<TypedExpr> {
    match &expr.expr {
        Expr::Var(v) => {
            if from.contains(&v) {
                Err("cycle detected".into())
            } else {
                match env.get(v) {
                    None => Err("unknown identifier".into()),
                    Some(e) => {
                        let mut path = from.clone();
                        path.push(v.clone());
                        resolve_expr(env, path, e)
                    }
                }
            }
        }
        Expr::Prim(_) => Ok(expr.clone()),
        Expr::Rel(rel) => {
            let uri = resolve_expr(env, from.clone(), &rel.uri);
            let methods = rel.methods.clone();
            let range = resolve_expr(env, from, &rel.range);

            uri.and_then(|uri| {
                range.and_then(|range| {
                    Ok(TypedExpr {
                        tag: expr.tag,
                        expr: Expr::Rel(Rel {
                            uri: Box::new(uri),
                            methods,
                            range: Box::new(range),
                        }),
                    })
                })
            })
        }
        Expr::Uri(uri) => {
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

            spec.map(|spec| TypedExpr {
                tag: expr.tag,
                expr: Expr::Uri(Uri { spec }),
            })
        }
        Expr::Block(block) => {
            let props: Result<Vec<_>> = block
                .iter()
                .map(|p| resolve_prop(env, from.clone(), p))
                .collect();

            props.map(|props| TypedExpr {
                tag: expr.tag,
                expr: Expr::Block(Block { props }),
            })
        }
        Expr::Sum(sum) => {
            let exprs: Result<Vec<_>> = sum
                .iter()
                .map(|e| resolve_expr(env, from.clone(), e))
                .collect();

            exprs.map(|exprs| TypedExpr {
                tag: expr.tag,
                expr: Expr::Sum(Sum { exprs }),
            })
        }
        Expr::Join(join) => {
            let exprs: Result<Vec<_>> = join
                .iter()
                .map(|e| resolve_expr(env, from.clone(), e))
                .collect();

            exprs.map(|exprs| TypedExpr {
                tag: expr.tag,
                expr: Expr::Join(Join { exprs }),
            })
        }
    }
}

pub fn resolve(env: &Scope, expr: &TypedExpr) -> Result<TypedExpr> {
    resolve_expr(env, Default::default(), expr)
}
