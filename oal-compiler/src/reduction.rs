use crate::errors::{Error, Result};
use crate::scope::Env;
use crate::transform::Transform;
use oal_syntax::ast::{Expr, Node};

pub fn reduce<T: Node>(acc: &mut (), env: &mut Env<T>, e: &mut T) -> Result<()> {
    e.as_mut().transform(acc, env, reduce)?;
    match e.as_mut() {
        Expr::Var(var) => match env.lookup(var) {
            None => Err(Error::new("identifier not in scope").with_expr(e.as_ref())),
            Some(val) => {
                match val.as_ref() {
                    Expr::Binding(_) => {}
                    _ => *e = val.clone(),
                };
                Ok(())
            }
        },
        Expr::App(application) => match env.lookup(&application.name) {
            None => Err(Error::new("identifier not in scope").with_expr(e.as_ref())),
            Some(val) => {
                if let Expr::Lambda(lambda) = val.as_ref() {
                    let app_env = &mut Env::new();
                    for (binding, arg) in lambda.bindings.iter().zip(application.args.iter()) {
                        if let Expr::Binding(name) = binding.as_ref() {
                            app_env.declare(name, arg)
                        } else {
                            unreachable!()
                        }
                    }
                    let mut app = lambda.body.as_ref().clone();
                    app.as_mut().transform(&mut (), app_env, reduce)?;
                    *e = app;
                    Ok(())
                } else {
                    Err(Error::new("identifier not a function").with_expr(e.as_ref()))
                }
            }
        },
        _ => Ok(()),
    }
}
