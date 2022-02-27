use crate::errors::{Error, Result};
use crate::transform::Transform;
use crate::Env;
use oal_syntax::ast::*;

pub fn compile(acc: &mut (), env: &mut Env, e: &mut TypedExpr) -> Result<()> {
    match &mut e.expr {
        Expr::Prim(_) => Ok(()),
        Expr::Rel(rel) => rel.transform(acc, env, compile),
        Expr::Uri(uri) => uri.transform(acc, env, compile),
        Expr::Block(block) => block.transform(acc, env, compile),
        Expr::Op(operation) => operation.transform(acc, env, compile),
        Expr::Var(var) => match env.lookup(var) {
            None => Err(Error::new("identifier not in scope")),
            Some(val) => {
                *e = val.clone();
                Ok(())
            }
        },
    }
}
