use crate::scope::Env;
use oal_syntax::ast::*;

pub trait Scan<T: Node> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: F) -> Result<(), E>
    where
        Self: Sized,
        E: Sized,
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>;
}

impl<T: Node> Scan<T> for Declaration<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        f(acc, env, &self.expr)?;
        env.declare(&self.name, &self.expr);
        Ok(())
    }
}

impl<T: Node> Scan<T> for Resource<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        f(acc, env, &self.rel)
    }
}

impl<T: Node> Scan<T> for Statement<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        match self {
            Statement::Decl(d) => d.scan(acc, env, f),
            Statement::Res(r) => r.scan(acc, env, f),
            Statement::Ann(_) => Ok(()),
        }
    }
}

impl<T: Node> Scan<T> for Program<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        env.within(|env| {
            self.into_iter()
                .try_for_each(|s| s.scan(acc, env, |a, v, e| f(a, v, e)))
        })
    }
}

impl<T: Node> Scan<T> for Relation<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Scan<T> for Uri<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Scan<T> for Object<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Scan<T> for Array<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Scan<T> for VariadicOp<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Scan<T> for Lambda<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        env.within(|env| {
            (&self.bindings)
                .into_iter()
                .try_for_each(|binding| {
                    f(acc, env, binding).and_then(|_| {
                        if let Expr::Binding(name) = binding.as_ref() {
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

impl<T: Node> Scan<T> for Application<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Scan<T> for Expr<T> {
    fn scan<F, E, U>(&self, acc: &mut U, env: &mut Env<T>, f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &T) -> Result<(), E>,
    {
        match self {
            Expr::Rel(rel) => rel.scan(acc, env, f),
            Expr::Uri(uri) => uri.scan(acc, env, f),
            Expr::Object(obj) => obj.scan(acc, env, f),
            Expr::Array(array) => array.scan(acc, env, f),
            Expr::Op(operation) => operation.scan(acc, env, f),
            Expr::Lambda(lambda) => lambda.scan(acc, env, f),
            Expr::App(application) => application.scan(acc, env, f),
            Expr::Prim(_) | Expr::Var(_) | Expr::Binding(_) => Ok(()),
        }
    }
}
