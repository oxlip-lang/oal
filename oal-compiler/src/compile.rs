use crate::errors::{Error, Result};
use crate::scope::Env;
use crate::transform::Transform;
use oal_syntax::ast::*;

pub fn compile(acc: &mut (), env: &mut Env, e: &mut TypedExpr) -> Result<()> {
    e.inner.transform(acc, env, compile)?;
    match &mut e.inner {
        Expr::Var(var) => match env.lookup(var) {
            None => Err(Error::new("identifier not in scope").with_expr(&e.inner)),
            Some(val) => {
                match val.inner {
                    Expr::Binding(_) => {}
                    _ => *e = val.clone(),
                };
                Ok(())
            }
        },
        Expr::App(application) => match env.lookup(&application.name) {
            None => Err(Error::new("identifier not in scope").with_expr(&e.inner)),
            Some(val) => {
                if let Expr::Lambda(lambda) = &val.inner {
                    let app_env = &mut Env::new();
                    for (binding, arg) in lambda.bindings.iter().zip(application.args.iter()) {
                        if let Expr::Binding(name) = &binding.inner {
                            app_env.declare(name, arg)
                        } else {
                            unreachable!()
                        }
                    }
                    let mut app = lambda.body.as_ref().clone();
                    app.inner.transform(&mut (), app_env, compile)?;
                    *e = app;
                    Ok(())
                } else {
                    Err(Error::new("identifier not a function").with_expr(&e.inner))
                }
            }
        },
        _ => Ok(()),
    }
}
