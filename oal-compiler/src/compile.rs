use crate::errors::{Error, Result};
use crate::transform::Transform;
use crate::Env;
use oal_syntax::ast::*;

pub fn compile(env: &mut Env, expr: &Expr) -> Result<Expr> {
    match expr {
        Expr::Prim(_) => Ok(expr.clone()),
        Expr::Rel(rel) => rel.transform(env, compile).map(Expr::Rel),
        Expr::Uri(uri) => uri.transform(env, compile).map(Expr::Uri),
        Expr::Block(block) => block.transform(env, compile).map(Expr::Block),
        Expr::Var(var) => {
            if let Some(e) = env.lookup(var) {
                Ok(e.expr.clone())
            } else {
                Err(Error::new("unknown variable"))
            }
        }
        Expr::Op(op) => op.transform(env, compile).map(Expr::Op),
    }
}
