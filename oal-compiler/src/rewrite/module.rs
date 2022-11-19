use crate::rewrite::tree::{NRef, Tree};
use crate::Locator;
use oal_model::grammar::NodeIdx;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Module {
    loc: Locator,
    tree: Tree,
}

impl Module {
    pub fn new(loc: Locator, tree: Tree) -> Self {
        Module { loc, tree }
    }

    pub fn locator(&self) -> &Locator {
        &self.loc
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }
}

#[derive(Debug)]
pub struct ModuleSet {
    base: Locator,
    mods: HashMap<Locator, Module>,
}

impl ModuleSet {
    pub fn new(base: Module) -> Self {
        ModuleSet {
            base: base.locator().clone(),
            mods: HashMap::from([(base.locator().clone(), base)]),
        }
    }

    pub fn base(&self) -> &Locator {
        &self.base
    }

    pub fn main(&self) -> &Module {
        self.mods.get(&self.base).unwrap()
    }

    pub fn insert(&mut self, m: Module) {
        self.mods.insert(m.locator().clone(), m);
    }

    pub fn len(&self) -> usize {
        self.mods.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, l: &Locator) -> Option<&Module> {
        self.mods.get(l)
    }
}

#[derive(Clone)]
pub struct External {
    loc: Locator,
    index: NodeIdx,
}

impl External {
    pub fn new(module: &Module, node: NRef) -> Self {
        External {
            loc: module.locator().clone(),
            index: node.index(),
        }
    }

    pub fn node<'a>(&self, mods: &'a ModuleSet) -> NRef<'a> {
        if let Some(module) = mods.get(&self.loc) {
            NRef::from(module.tree(), self.index)
        } else {
            // All modules must be present in the module-set.
            panic!("unknown module: {}", self.loc)
        }
    }
}

impl std::fmt::Display for External {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}#{}", &self.loc, &self.index.to_string())
    }
}

impl std::fmt::Debug for External {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(&self, f)
    }
}
