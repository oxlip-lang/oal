use crate::errors::Error;
use crate::node::NodeMut;
use crate::scope::Env;
use oal_syntax::ast::*;

pub trait Transform<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        Self: Sized,
        E: Sized + From<Error>,
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>;
}

fn transform_expr<T, F, E, U>(e: &mut T, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
where
    T: AsExpr,
    E: From<Error>,
    F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
{
    e.as_node_mut().as_expr_mut().transform(acc, env, f)?;
    f(acc, env, NodeMut::Expr(e))
}

macro_rules! transform_expr_node {
    ( $node:ident ) => {
        impl<T: AsExpr> Transform<T> for $node<T> {
            fn transform<F, E, U>(
                &mut self,
                acc: &mut U,
                env: &mut Env<T>,
                f: &mut F,
            ) -> Result<(), E>
            where
                E: From<Error>,
                F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
            {
                self.into_iter()
                    .try_for_each(|e| transform_expr(e, acc, env, f))
            }
        }
    };
}

transform_expr_node!(Relation);
transform_expr_node!(Uri);
transform_expr_node!(Property);
transform_expr_node!(Object);
transform_expr_node!(Content);
transform_expr_node!(Transfer);
transform_expr_node!(Array);
transform_expr_node!(VariadicOp);
transform_expr_node!(Application);

impl<T: AsExpr> Transform<T> for Declaration<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        E: From<Error>,
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        f(acc, env, NodeMut::Decl(self))?;
        self.into_iter()
            .try_for_each(|e| transform_expr(e, acc, env, f))?;
        env.declare(self.name.clone(), self.expr.clone());
        Ok(())
    }
}

impl<T: AsExpr> Transform<T> for Resource<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        E: From<Error>,
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        f(acc, env, NodeMut::Res(self))?;
        self.into_iter()
            .try_for_each(|e| transform_expr(e, acc, env, f))
    }
}

impl<T> Transform<T> for Annotation {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        E: From<Error>,
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        f(acc, env, NodeMut::Ann(self))
    }
}

impl<T: AsExpr> Transform<T> for Import {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        E: From<Error>,
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        f(acc, env, NodeMut::Use(self))?;
        env.import(self.module.as_str())?;
        Ok(())
    }
}

impl<T: AsExpr> Transform<T> for Statement<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        E: From<Error>,
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        match self {
            Statement::Decl(d) => d.transform(acc, env, f),
            Statement::Res(r) => r.transform(acc, env, f),
            Statement::Ann(a) => a.transform(acc, env, f),
            Statement::Use(u) => u.transform(acc, env, f),
        }
    }
}

impl<T: AsExpr> Transform<T> for Program<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        E: From<Error>,
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        env.within(|env| self.into_iter().try_for_each(|s| s.transform(acc, env, f)))
    }
}

impl<T: AsExpr> Transform<T> for Lambda<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        E: From<Error>,
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        env.within(|env| {
            self.bindings
                .iter_mut()
                .try_for_each(|binding| {
                    transform_expr(binding, acc, env, f).and_then(|_| {
                        if let Expr::Binding(name) = binding.as_node().as_expr() {
                            env.declare(name.clone(), binding.clone());
                            Ok(())
                        } else {
                            unreachable!()
                        }
                    })
                })
                .and_then(|_| transform_expr(self.body.as_mut(), acc, env, f))
        })
    }
}

impl<T: AsExpr> Transform<T> for Expr<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        E: From<Error>,
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        match self {
            Expr::Rel(rel) => rel.transform(acc, env, f),
            Expr::Uri(uri) => uri.transform(acc, env, f),
            Expr::Property(prop) => prop.transform(acc, env, f),
            Expr::Object(obj) => obj.transform(acc, env, f),
            Expr::Content(cnt) => cnt.transform(acc, env, f),
            Expr::Xfer(xfer) => xfer.transform(acc, env, f),
            Expr::Array(array) => array.transform(acc, env, f),
            Expr::Op(operation) => operation.transform(acc, env, f),
            Expr::Lambda(lambda) => lambda.transform(acc, env, f),
            Expr::App(application) => application.transform(acc, env, f),
            Expr::Lit(_) | Expr::Prim(_) | Expr::Var(_) | Expr::Binding(_) => Ok(()),
        }
    }
}
