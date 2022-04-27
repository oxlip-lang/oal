use crate::scope::Env;
use oal_syntax::ast::*;

pub trait Transform<T: Node> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: F) -> Result<(), E>
    where
        Self: Sized,
        E: Sized,
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>;
}

impl<T: Node> Transform<T> for Declaration<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        f(acc, env, &mut self.expr)?;
        env.declare(&self.name, &self.expr);
        Ok(())
    }
}

impl<T: Node> Transform<T> for Resource<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Transform<T> for Statement<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        match self {
            Statement::Decl(d) => d.transform(acc, env, f),
            Statement::Res(r) => r.transform(acc, env, f),
            Statement::Ann(_) => Ok(()),
        }
    }
}

impl<T: Node> Transform<T> for Program<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        env.within(|env| {
            self.into_iter()
                .try_for_each(|s| s.transform(acc, env, |a, v, e| f(a, v, e)))
        })
    }
}

impl<T: Node> Transform<T> for Relation<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Transform<T> for Uri<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Transform<T> for Object<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Transform<T> for Array<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Transform<T> for VariadicOp<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Transform<T> for Lambda<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        env.within(|env| {
            (&mut self.bindings)
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
                .and_then(|_| f(acc, env, &mut self.body))
        })
    }
}

impl<T: Node> Transform<T> for Application<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, mut f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        self.into_iter().try_for_each(|e| f(acc, env, e))
    }
}

impl<T: Node> Transform<T> for Expr<T> {
    fn transform<F, E, U>(&mut self, acc: &mut U, env: &mut Env<T>, f: F) -> Result<(), E>
    where
        F: FnMut(&mut U, &mut Env<T>, &mut T) -> Result<(), E>,
    {
        match self {
            Expr::Rel(rel) => rel.transform(acc, env, f),
            Expr::Uri(uri) => uri.transform(acc, env, f),
            Expr::Object(obj) => obj.transform(acc, env, f),
            Expr::Array(array) => array.transform(acc, env, f),
            Expr::Op(operation) => operation.transform(acc, env, f),
            Expr::Lambda(lambda) => lambda.transform(acc, env, f),
            Expr::App(application) => application.transform(acc, env, f),
            Expr::Prim(_) | Expr::Var(_) | Expr::Binding(_) => Ok(()),
        }
    }
}
