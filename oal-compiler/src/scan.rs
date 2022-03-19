use crate::Env;
use oal_syntax::ast::*;
use oal_syntax::try_each::TryEach;

pub trait Scan {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, f: F) -> Result<(), E>
    where
        Self: Sized,
        E: Sized,
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>;
}

impl Scan for Decl {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        f(acc, env, &self.expr)?;
        env.declare(&self.name, &self.expr);
        Ok(())
    }
}

impl Scan for Res {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        f(acc, env, &self.rel)
    }
}

impl Scan for Stmt {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        match self {
            Stmt::Decl(d) => d.scan(acc, env, f),
            Stmt::Res(r) => r.scan(acc, env, f),
        }
    }
}

impl Scan for Doc {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        env.within(|env| self.try_each(|s| s.scan(acc, env, |a, v, e| f(a, v, e))))
    }
}

impl Scan for Rel {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        self.try_each(|e| f(acc, env, e))
    }
}

impl Scan for Uri {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        self.try_each(|e| f(acc, env, e))
    }
}

impl Scan for Block {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        self.try_each(|e| f(acc, env, e))
    }
}

impl Scan for Array {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        self.try_each(|e| f(acc, env, e))
    }
}

impl Scan for VariadicOp {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        self.try_each(|e| f(acc, env, e))
    }
}

impl Scan for Lambda {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        env.within(|env| {
            (&self.bindings)
                .try_each(|binding| {
                    f(acc, env, binding).and_then(|_| {
                        if let Expr::Binding(name) = &binding.inner {
                            env.declare(name, binding);
                            Ok(())
                        } else {
                            unreachable!()
                        }
                    })
                })
                .and_then(|_| f(acc, env, &self.body))
        })
    }
}

impl Scan for Application {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        self.try_each(|e| f(acc, env, e))
    }
}

impl Scan for Expr {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env, f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &TypedExpr) -> Result<(), E>,
    {
        match self {
            Expr::Rel(rel) => rel.scan(acc, env, f),
            Expr::Uri(uri) => uri.scan(acc, env, f),
            Expr::Block(block) => block.scan(acc, env, f),
            Expr::Array(array) => array.scan(acc, env, f),
            Expr::Op(operation) => operation.scan(acc, env, f),
            Expr::Lambda(lambda) => lambda.scan(acc, env, f),
            Expr::App(application) => application.scan(acc, env, f),
            Expr::Prim(_) | Expr::Var(_) | Expr::Binding(_) => Ok(()),
        }
    }
}
