use crate::inference::tag::Tag;
use crate::module::{External, ModuleSet};
use oal_model::grammar::{NodeRef, SyntaxTree};
use oal_syntax::parser::Gram;

/// The internally mutable `Core` type for the syntax tree.
#[derive(Clone, Default, Debug)]
pub struct Core {
    defn: Option<External>,
    tag: Option<Tag>,
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

    pub fn tag(&self) -> Option<&Tag> {
        self.tag.as_ref()
    }

    pub fn set_tag(&mut self, t: Tag) {
        self.tag.replace(t);
    }

    pub fn unwrap_tag(&self) -> Tag {
        self.tag.as_ref().expect("tag missing").clone()
    }

    pub fn with_tag(mut self, t: Tag) -> Self {
        self.tag = Some(t);
        self
    }
}

/// The syntax tree type.
pub type Tree = SyntaxTree<Core, Gram>;

/// The node reference type.
pub type NRef<'a> = NodeRef<'a, Core, Gram>;

/// Returns the definition of the given node if any.
pub fn definition<'a>(mods: &'a ModuleSet, node: NRef<'a>) -> Option<NRef<'a>> {
    node.syntax().core_ref().definition().map(|n| n.node(mods))
}

/// Returns the type tag for the given node.
pub fn get_tag(n: NRef) -> Tag {
    n.syntax().core_ref().unwrap_tag()
}

/// Sets the type tag for the given node.
pub fn set_tag(n: NRef, t: Tag) {
    n.syntax().core_mut().set_tag(t)
}
