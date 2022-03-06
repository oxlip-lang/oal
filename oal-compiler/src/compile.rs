use crate::errors::{Error, Result};
use crate::transform::Transform;
use crate::Env;
use oal_syntax::ast::*;

pub fn compile(acc: &mut (), env: &mut Env, e: &mut TypedExpr) -> Result<()> {
    e.inner.transform(acc, env, compile)?;
    match &mut e.inner {
        Expr::Var(var) => match env.lookup(var) {
            None => Err(Error::new("identifier not in scope")
                .with_expr(&e.inner)
                .with_tag(&e.tag)),
            Some(val) => {
                *e = val.clone();
                Ok(())
            }
        },
        Expr::App(_) => todo!(),
        _ => Ok(()),
    }
}
