use crate::scope::Env;
use oal_syntax::ast::*;

pub trait Scan<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        Self: Sized,
        E: Sized,
        F: FnMut(&mut U, &mut Env<T>, NodeRef<T>) -> Result<(), E>;
}

fn scan_expr<T, F, E, U>(e: &T, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
where
    T: AsExpr,
    F: FnMut(&mut U, &mut Env<T>, NodeRef<T>) -> Result<(), E>,
{
    e.as_node().as_expr().scan(acc, env, f)?;
    f(acc, env, NodeRef::Expr(e))
}

macro_rules! scan_expr_node {
    ( $node:ident ) => {
        impl<T: AsExpr> Scan<T> for $node<T> {
            fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
            where
                F: FnMut(&mut U, &mut Env<T>, NodeRef<T>) -> Result<(), E>,
            {
                self.into_iter().try_for_each(|e| scan_expr(e, acc, env, f))
            }
        }
    };
}

scan_expr_node!(Relation);
scan_expr_node!(Uri);
scan_expr_node!(Object);
scan_expr_node!(Content);
scan_expr_node!(Transfer);
scan_expr_node!(Array);
scan_expr_node!(VariadicOp);
scan_expr_node!(Application);

impl<T: AsExpr> Scan<T> for Declaration<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeRef<T>) -> Result<(), E>,
    {
        f(acc, env, NodeRef::Decl(self))?;
        self.into_iter()
            .try_for_each(|e| scan_expr(e, acc, env, f))?;
        env.declare(&self.name, &self.expr);
        Ok(())
    }
}

impl<T: AsExpr> Scan<T> for Resource<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeRef<T>) -> Result<(), E>,
    {
        f(acc, env, NodeRef::Res(self))?;
        self.into_iter().try_for_each(|e| scan_expr(e, acc, env, f))
    }
}

impl<T> Scan<T> for Annotation {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeRef<T>) -> Result<(), E>,
    {
        f(acc, env, NodeRef::Ann(self))
    }
}

impl<T: AsExpr> Scan<T> for Statement<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeRef<T>) -> Result<(), E>,
    {
        match self {
            Statement::Decl(d) => d.scan(acc, env, f),
            Statement::Res(r) => r.scan(acc, env, f),
            Statement::Ann(a) => a.scan(acc, env, f),
        }
    }
}

impl<T: AsExpr> Scan<T> for Program<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeRef<T>) -> Result<(), E>,
    {
        env.within(|env| self.into_iter().try_for_each(|s| s.scan(acc, env, f)))
    }
}

impl<T: AsExpr> Scan<T> for Lambda<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeRef<T>) -> Result<(), E>,
    {
        env.within(|env| {
            (&self.bindings)
                .into_iter()
                .try_for_each(|binding| {
                    scan_expr(binding, acc, env, f).and_then(|_| {
                        if let Expr::Binding(name) = binding.as_node().as_expr() {
                            env.declare(name, binding);
                            Ok(())
                        } else {
                            unreachable!()
                        }
                    })
                })
                .and_then(|_| scan_expr(self.body.as_ref(), acc, env, f))
        })
    }
}

impl<T: AsExpr> Scan<T> for Expr<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, NodeRef<T>) -> Result<(), E>,
    {
        match self {
            Expr::Rel(rel) => rel.scan(acc, env, f),
            Expr::Uri(uri) => uri.scan(acc, env, f),
            Expr::Object(obj) => obj.scan(acc, env, f),
            Expr::Content(cnt) => cnt.scan(acc, env, f),
            Expr::Xfer(xfer) => xfer.scan(acc, env, f),
            Expr::Array(array) => array.scan(acc, env, f),
            Expr::Op(operation) => operation.scan(acc, env, f),
            Expr::Lambda(lambda) => lambda.scan(acc, env, f),
            Expr::App(application) => application.scan(acc, env, f),
            Expr::Prim(_) | Expr::Var(_) | Expr::Binding(_) => Ok(()),
        }
    }
}
