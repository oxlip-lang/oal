use crate::definition::Definition;
use oal_syntax::atom::Ident;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Entry(Ident, Option<Ident>);

impl Entry {
    pub fn new(ident: Ident, qualifier: Option<Ident>) -> Self {
        Entry(ident, qualifier)
    }
}

impl From<Ident> for Entry {
    fn from(i: Ident) -> Self {
        Entry(i, None)
    }
}

pub type Scope = HashMap<Entry, Definition>;

#[derive(Debug)]
pub struct Env(Vec<Scope>);

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

impl Env {
    pub fn new() -> Self {
        Env(vec![Scope::new()])
    }

    pub fn declare(&mut self, e: Entry, defn: Definition) -> Option<Definition> {
        self.0.last_mut().unwrap().insert(e, defn)
    }

    pub fn lookup(&self, e: &Entry) -> Option<&Definition> {
        self.0
            .iter()
            .rev()
            .map(|s| s.get(e))
            .skip_while(Option::is_none)
            .map(|s| s.unwrap())
            .next()
    }

    pub fn open(&mut self) {
        self.0.push(Scope::new());
    }

    pub fn close(&mut self) {
        self.0.pop();
    }
}
