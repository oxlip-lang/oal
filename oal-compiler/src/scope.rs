use oal_syntax::ast::{Ident, TypedExpr};
use std::collections::HashMap;

pub type Scope = HashMap<Ident, TypedExpr>;

pub struct Env {
    scopes: Vec<Scope>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            scopes: vec![Scope::new()],
        }
    }
    pub fn head(&self) -> &Scope {
        self.scopes.last().unwrap()
    }
    pub fn open(&mut self) {
        self.scopes.push(Scope::new());
    }
    pub fn declare(&mut self, n: &Ident, e: &TypedExpr) {
        self.scopes.last_mut().unwrap().insert(n.clone(), e.clone());
    }
    pub fn lookup(&self, n: &Ident) -> Option<&TypedExpr> {
        self.scopes
            .iter()
            .rev()
            .map(|s| s.get(n))
            .skip_while(Option::is_none)
            .map(|s| s.unwrap())
            .next()
    }
    pub fn exists(&self, n: &Ident) -> bool {
        self.scopes.last().unwrap().contains_key(n)
    }
    pub fn close(&mut self) {
        self.scopes.pop();
    }
}
