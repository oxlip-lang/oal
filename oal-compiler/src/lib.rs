mod errors;
mod inference;
mod resolver;
mod scope;

#[cfg(test)]
mod inference_tests;
#[cfg(test)]
mod scope_tests;

use crate::errors::Result;
use crate::resolver::resolve;
use crate::scope::Env;
use oal_syntax::ast::{Doc, Expr, Rel, Stmt};

fn global_env(d: &Doc) -> Env {
    let mut e = Env::new();
    for s in d.stmts.iter() {
        if let Stmt::Decl(d) = s {
            e.declare(&d.var, &d.body)
        }
    }
    e
}

pub fn relations(doc: &Doc) -> Result<Vec<Rel>> {
    let env = global_env(doc);

    doc.stmts
        .iter()
        .flat_map(|s| match s {
            Stmt::Res(r) => Some(&r.rel),
            _ => None,
        })
        .map(|e| {
            resolve(env.head(), e).and_then(|e| match e.expr {
                Expr::Rel(rel) => Ok(rel),
                _ => Err("expected relation".into()),
            })
        })
        .collect()
}
