use crate::rewrite::module::External;
use oal_model::grammar::{NodeRef, SyntaxTree};
use oal_syntax::rewrite::parser::Gram;

/// The internally mutable `Core` type for the syntax tree.
#[derive(Clone, Default, Debug)]
pub struct Core {
    defn: Option<External>,
}

impl Core {
    /// Returns the definition of the current node if any.
    pub fn definition(&self) -> Option<&External> {
        self.defn.as_ref()
    }

    /// Sets the location of the definition for the current node.
    pub fn define(&mut self, ext: External) {
        self.defn = Some(ext);
    }
}

/// The syntax tree type.
pub type Tree = SyntaxTree<Core, Gram>;

/// The node reference type.
pub type NRef<'a> = NodeRef<'a, Core, Gram>;
