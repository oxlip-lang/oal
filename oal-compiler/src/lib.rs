mod environment;
mod errors;
mod resolver;
mod type_checker;

use crate::environment::environment;
use crate::errors::Result;
use crate::resolver::resolve;
use crate::type_checker::{well_type, TypeTag};
use oal_syntax::ast::{Doc, Stmt};

pub struct Paths {
    _paths: Vec<openapiv3::PathItem>,
}

pub fn paths(d: &Doc) -> Result<Paths> {
    let env = environment(&d);

    let e: Result<Vec<_>> = d
        .stmts
        .iter()
        .flat_map(|s| match s {
            Stmt::Res(r) => Some(&r.rel),
            _ => None,
        })
        .map(|e| {
            resolve(&env, e).and_then(|e| {
                well_type(&e).and_then(|t| {
                    if t == TypeTag::Rel {
                        Ok(e)
                    } else {
                        Err("expected relation".into())
                    }
                })
            })
        })
        .collect();

    println!("{:#?}", e);

    Ok(Paths { _paths: vec![] })
}
