use crate::errors::Result;
use crate::eval;
use crate::inference::tag::{Seq, Tag};
use crate::module::ModuleSet;
use crate::tree::{NRef, Tree};
use oal_model::grammar::NodeIdx;
use oal_model::locator::Locator;
use std::fmt::Debug;
use std::rc::Rc;

/// Internal identifier definition.
pub trait Internal: Debug {
    fn tag(&self, seq: &mut Seq) -> Tag;
    fn eval<'a>(&self, args: Vec<eval::Value<'a>>, ann: eval::AnnRef) -> Result<eval::Value<'a>>;
    fn has_bindings(&self) -> bool;
}

pub type InternalRef = Rc<dyn Internal>;

/// External identifier definition.
#[derive(Clone)]
pub struct External {
    loc: Locator,
    index: NodeIdx,
}

impl External {
    pub fn new(module: &Tree, node: NRef) -> Self {
        External {
            loc: module.locator().clone(),
            index: node.index(),
        }
    }

    pub fn node<'a>(&self, mods: &'a ModuleSet) -> NRef<'a> {
        if let Some(module) = mods.get(&self.loc) {
            NRef::from(module, self.index)
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
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

/// Identifier definition.
#[derive(Debug, Clone)]
pub enum Definition {
    External(External),
    Internal(InternalRef),
}
