use oal_syntax::ast::{Doc, Ident, Stmt, TypeExpr};
use std::collections::HashMap;

pub type Env = HashMap<Ident, TypeExpr>;

pub fn environment(d: &Doc) -> Env {
    d.stmts
        .iter()
        .flat_map(|s| match s {
            Stmt::Decl(d) => Some((d.var.clone(), d.expr.clone())),
            _ => None,
        })
        .collect()
}
