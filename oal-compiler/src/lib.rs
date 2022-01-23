mod errors;
mod inference;
mod resolver;
mod scope;
mod type_checker;

#[cfg(test)]
mod scope_tests;

use crate::errors::Result;
use crate::resolver::resolve;
use crate::scope::Env;
use crate::type_checker::well_type;
use oal_syntax::ast::{Doc, Stmt, TypeExpr, TypeRel, TypeTag};

fn global_env(d: &Doc) -> Env {
    let mut e = Env::new();
    for s in d.stmts.iter() {
        if let Stmt::Decl(d) = s {
            e.declare(&d.var, &d.expr)
        }
    }
    e
}

pub fn relations(doc: &Doc) -> Result<Vec<TypeRel>> {
    let env = global_env(doc);

    doc.stmts
        .iter()
        .flat_map(|s| match s {
            Stmt::Res(r) => Some(&r.rel),
            _ => None,
        })
        .map(|e| {
            resolve(env.head(), e).and_then(|e| {
                well_type(&e).and_then(|t| match e {
                    TypeExpr::Rel(rel) if t == TypeTag::Relation => Ok(rel),
                    _ => Err("expected relation".into()),
                })
            })
        })
        .collect()
}
