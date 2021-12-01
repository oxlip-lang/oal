mod environment;
mod errors;
mod resolver;
mod type_checker;

use crate::environment::environment;
use crate::errors::Result;
use crate::resolver::resolve;
use crate::type_checker::{well_type, TypeTag};
use oal_syntax::ast::{Doc, Stmt, TypeExpr, TypeRel};

pub fn relations(doc: &Doc) -> Result<Vec<TypeRel>> {
    let env = environment(doc);

    doc.stmts
        .iter()
        .flat_map(|s| match s {
            Stmt::Res(r) => Some(&r.rel),
            _ => None,
        })
        .map(|e| {
            resolve(&env, e).and_then(|e| {
                well_type(&e).and_then(|t| match e {
                    TypeExpr::Rel(rel) if t == TypeTag::Rel => Ok(rel),
                    _ => Err("expected relation".into()),
                })
            })
        })
        .collect()
}
