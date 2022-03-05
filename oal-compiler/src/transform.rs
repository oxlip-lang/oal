use crate::Env;
use oal_syntax::ast::*;
use oal_syntax::try_each::TryEach;

pub trait Transform {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env, f: F) -> Result<(), E>
    where
        Self: Sized,
        E: Sized,
        F: FnMut(&mut U, &mut Env, &mut TypedExpr) -> Result<(), E>;
}

impl Transform for Decl {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &mut TypedExpr) -> Result<(), E>,
    {
        f(acc, env, &mut self.expr)?;
        env.declare(&self.name, &self.expr);
        Ok(())
    }
}

impl Transform for Res {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &mut TypedExpr) -> Result<(), E>,
    {
        f(acc, env, &mut self.rel)
    }
}

impl Transform for Stmt {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env, f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &mut TypedExpr) -> Result<(), E>,
    {
        match self {
            Stmt::Decl(d) => d.transform(acc, env, f),
            Stmt::Res(r) => r.transform(acc, env, f),
        }
    }
}

impl Transform for Doc {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &mut TypedExpr) -> Result<(), E>,
    {
        env.open();
        let r = self.try_each(|s| s.transform(acc, env, |a, v, e| f(a, v, e)));
        env.close();
        r
    }
}

impl Transform for Rel {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &mut TypedExpr) -> Result<(), E>,
    {
        self.try_each(|e| f(acc, env, e))
    }
}

impl Transform for Uri {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &mut TypedExpr) -> Result<(), E>,
    {
        self.try_each(|e| f(acc, env, e))
    }
}

impl Transform for Block {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &mut TypedExpr) -> Result<(), E>,
    {
        self.try_each(|e| f(acc, env, e))
    }
}

impl Transform for VariadicOp {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &mut TypedExpr) -> Result<(), E>,
    {
        self.try_each(|e| f(acc, env, e))
    }
}

impl Transform for Lambda {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &mut TypedExpr) -> Result<(), E>,
    {
        env.open();

        let r = (&mut self.bindings).try_each(|binding| {
            f(acc, env, binding).and_then(|_| {
                if let Expr::Binding(name) = &binding.inner {
                    env.declare(name, binding);
                    Ok(())
                } else {
                    unreachable!()
                }
            })
        });

        let r = r.and_then(|_| f(acc, env, &mut self.body));

        env.close();

        r
    }
}

impl Transform for Application {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env, &mut TypedExpr) -> Result<(), E>,
    {
        self.try_each(|e| f(acc, env, e))
    }
}
