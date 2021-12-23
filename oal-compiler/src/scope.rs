use oal_syntax::ast::{Ident, TypeExpr};
use std::collections::HashMap;

pub type Scope = HashMap<Ident, TypeExpr>;

pub struct Env(Vec<Scope>);

impl Env {
    pub fn new() -> Env {
        Env(vec![Scope::new()])
    }
    pub fn head(&self) -> &Scope {
        self.0.last().unwrap()
    }
    pub fn open(&mut self) {
        self.0.push(Scope::new());
    }
    pub fn declare(&mut self, n: &Ident, e: &TypeExpr) {
        self.0.last_mut().unwrap().insert(n.clone(), e.clone());
    }
    pub fn lookup(&self, n: &Ident) -> Option<&TypeExpr> {
        self.0
            .iter()
            .rev()
            .map(|s| s.get(n))
            .skip_while(Option::is_none)
            .map(|s| s.unwrap())
            .next()
    }
    pub fn exists(&self, n: &Ident) -> bool {
        self.0.last().unwrap().contains_key(n)
    }
    pub fn close(&mut self) {
        self.0.pop();
    }
}
