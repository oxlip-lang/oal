use crate::errors::{Error, Kind, Result};
use crate::scope::Env;
use crate::transform::Transform;
use oal_syntax::ast::{AsExpr, Expr, NodeMut};

pub trait Semigroup: Sized {
    fn combine(&mut self, with: Self) {
        *self = with;
    }
}

pub fn reduce<T>(_acc: &mut (), env: &mut Env<T>, node: NodeMut<T>) -> Result<()>
where
    T: AsExpr + Semigroup,
{
    match node {
        NodeMut::Expr(e) => match e.as_node_mut().as_expr_mut() {
            Expr::Var(var) => match env.lookup(var) {
                None => Err(Error::new(Kind::IdentifierNotInScope, "").with(e)),
                Some(val) => {
                    match val.as_node().as_expr() {
                        Expr::Binding(_) => {}
                        _ => e.combine(val.clone()),
                    };
                    Ok(())
                }
            },
            Expr::App(application) => match env.lookup(&application.name) {
                None => Err(Error::new(Kind::IdentifierNotAFunction, "").with(e)),
                Some(val) => {
                    if let Expr::Lambda(lambda) = val.as_node().as_expr() {
                        let app_env = &mut Env::new(None);
                        for (binding, arg) in lambda.bindings.iter().zip(application.args.iter()) {
                            if let Expr::Binding(name) = binding.as_node().as_expr() {
                                app_env.declare(name.clone(), arg.clone())
                            } else {
                                unreachable!()
                            }
                        }
                        let mut app = lambda.body.as_ref().clone();
                        app.as_node_mut()
                            .as_expr_mut()
                            .transform(&mut (), app_env, &mut reduce)?;
                        e.combine(app);
                        Ok(())
                    } else {
                        Err(Error::new(Kind::IdentifierNotAFunction, "").with(e))
                    }
                }
            },
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}
