use crate::environment::Env;
use crate::errors::Result;
use oal_syntax::ast::*;

type Path = Vec<Ident>;

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
    resolve_expr(env, Default::default(), expr)
}
