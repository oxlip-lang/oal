use crate::scope::Env;
use oal_syntax::ast::*;

pub trait Transform<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        Self: Sized,
        E: Sized,
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>;
}

fn transform_expr<T, F, E, U>(e: &mut T, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
where
    T: AsExpr,
    F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
{
    e.as_mut().transform(acc, env, f)?;
    f(acc, env, NodeMut::Expr(e))
}

impl<T: AsExpr> Transform<T> for Declaration<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        f(acc, env, NodeMut::Decl(self))?;
        self.into_iter()
            .try_for_each(|e| transform_expr(e, acc, env, f))?;
        env.declare(&self.name, &self.expr);
        Ok(())
    }
}

impl<T: AsExpr> Transform<T> for Resource<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
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
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        f(acc, env, NodeMut::Ann(self))
    }
}

impl<T: AsExpr> Transform<T> for Statement<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        match self {
            Statement::Decl(d) => d.transform(acc, env, f),
            Statement::Res(r) => r.transform(acc, env, f),
            Statement::Ann(a) => a.transform(acc, env, f),
        }
    }
}

impl<T: AsExpr> Transform<T> for Program<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        env.within(|env| self.into_iter().try_for_each(|s| s.transform(acc, env, f)))
    }
}

impl<T: AsExpr> Transform<T> for Relation<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        self.into_iter()
            .try_for_each(|e| transform_expr(e, acc, env, f))
    }
}

impl<T: AsExpr> Transform<T> for Uri<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        self.into_iter()
            .try_for_each(|e| transform_expr(e, acc, env, f))
    }
}

impl<T: AsExpr> Transform<T> for Object<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        self.into_iter()
            .try_for_each(|e| transform_expr(e, acc, env, f))
    }
}

impl<T: AsExpr> Transform<T> for Content<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        self.into_iter()
            .try_for_each(|e| transform_expr(e, acc, env, f))
    }
}

impl<T: AsExpr> Transform<T> for Transfer<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        self.into_iter()
            .try_for_each(|e| transform_expr(e, acc, env, f))
    }
}

impl<T: AsExpr> Transform<T> for Array<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        self.into_iter()
            .try_for_each(|e| transform_expr(e, acc, env, f))
    }
}

impl<T: AsExpr> Transform<T> for VariadicOp<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        self.into_iter()
            .try_for_each(|e| transform_expr(e, acc, env, f))
    }
}

impl<T: AsExpr> Transform<T> for Lambda<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        env.within(|env| {
            (&mut self.bindings)
                .into_iter()
                .try_for_each(|binding| {
                    transform_expr(binding, acc, env, f).and_then(|_| {
                        if let Expr::Binding(name) = binding.as_ref() {
                            env.declare(name, binding);
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

impl<T: AsExpr> Transform<T> for Application<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        self.into_iter()
            .try_for_each(|e| transform_expr(e, acc, env, f))
    }
}

impl<T: AsExpr> Transform<T> for Expr<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeMut<T>) -> Result<(), E>,
    {
        match self {
            Expr::Rel(rel) => rel.transform(acc, env, f),
            Expr::Uri(uri) => uri.transform(acc, env, f),
            Expr::Object(obj) => obj.transform(acc, env, f),
            Expr::Content(cnt) => cnt.transform(acc, env, f),
            Expr::Xfer(xfer) => xfer.transform(acc, env, f),
            Expr::Array(array) => array.transform(acc, env, f),
            Expr::Op(operation) => operation.transform(acc, env, f),
            Expr::Lambda(lambda) => lambda.transform(acc, env, f),
            Expr::App(application) => application.transform(acc, env, f),
            Expr::Prim(_) | Expr::Var(_) | Expr::Binding(_) => Ok(()),
        }
    }
}
