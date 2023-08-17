use crate::errors::Result;
use crate::eval;
use crate::inference::tag::{Seq, Tag};
use crate::module::ModuleSet;
use crate::tree::NRef;
use oal_model::grammar::NodeIdx;
use oal_model::locator::Locator;
use sha2::{Digest, Sha256};
use std::fmt::{Debug, Formatter, LowerHex};
use std::rc::Rc;

/// Internal identifier definition.
pub trait Internal: Debug {
    fn tag(&self, seq: &mut Seq) -> Tag;
    fn eval<'a>(&self, args: Vec<eval::Value<'a>>, ann: eval::AnnRef) -> Result<eval::Value<'a>>;
    fn has_bindings(&self) -> bool;
    fn id(&self) -> u32;
}

impl PartialEq for dyn Internal {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for dyn Internal {}

pub type InternalRef = Rc<dyn Internal>;

/// External identifier definition.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct External {
    loc: Locator,
    index: NodeIdx,
}

impl External {
    pub fn new(node: NRef) -> Self {
        External {
            loc: node.tree().locator().clone(),
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

impl LowerHex for External {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let arena_index = generational_arena::Index::from(self.index);
        let (index, generation) = arena_index.into_raw_parts();
        let hash = Sha256::new()
            .chain_update(self.loc.url().as_str())
            .chain_update(index.to_be_bytes())
            .chain_update(generation.to_be_bytes())
            .finalize();
        write!(f, "{:x}", hash)
    }
}

/// Identifier definition.
#[derive(Debug, Clone)]
pub enum Definition {
    External(External),
    Internal(InternalRef),
}

impl PartialEq for Definition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::External(l), Self::External(r)) => l == r,
            (Self::Internal(l), Self::Internal(r)) => l == r,
            _ => false,
        }
    }
}

impl Eq for Definition {}
