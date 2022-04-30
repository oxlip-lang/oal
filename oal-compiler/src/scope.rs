use oal_syntax::ast::{AsExpr, Ident};
use std::collections::HashMap;

pub type Scope<T> = HashMap<Ident, T>;

pub struct Env<T> {
    scopes: Vec<Scope<T>>,
}

impl<T: AsExpr> Env<T> {
    pub fn new() -> Env<T> {
        Env {
            scopes: vec![Scope::new()],
        }
    }

    #[cfg(test)]
    pub fn head(&self) -> &Scope<T> {
        self.scopes.last().unwrap()
    }

    pub fn declare(&mut self, n: &Ident, e: &T) {
        self.scopes.last_mut().unwrap().insert(n.clone(), e.clone());
    }

    pub fn lookup(&self, n: &Ident) -> Option<&T> {
        self.scopes
            .iter()
            .rev()
            .map(|s| s.get(n))
            .skip_while(Option::is_none)
            .map(|s| s.unwrap())
            .next()
    }

    #[cfg(test)]
    pub fn exists(&self, n: &Ident) -> bool {
        self.scopes.last().unwrap().contains_key(n)
    }

    pub fn within<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(&mut Self) -> R,
    {
        self.open();
        let r = f(self);
        self.close();
        r
    }

    fn open(&mut self) {
        self.scopes.push(Scope::new());
    }
    fn close(&mut self) {
        self.scopes.pop();
    }
}
