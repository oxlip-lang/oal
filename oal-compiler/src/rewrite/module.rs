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
    pub fn new(base: Locator) -> Self {
        ModuleSet {
            base,
            mods: Default::default(),
        }
    }

    pub fn base(&self) -> &Locator {
        &self.base
    }

    pub fn main(&self) -> &Module {
        self.mods.get(&self.base).unwrap()
    }

    pub fn insert(&mut self, l: Locator, m: Module) {
        self.mods.insert(l, m);
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

#[derive(Clone, Debug)]
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
