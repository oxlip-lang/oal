use crate::definition::Definition;
use crate::inference::tag::Tag;
use oal_model::grammar::{NodeRef, SyntaxTree};
use oal_syntax::parser::Gram;

/// The internally mutable `Core` type for the syntax tree.
#[derive(Clone, Default, Debug)]
pub struct Core {
    defn: Option<Definition>,
    tag: Option<Tag>,
}

impl Core {
    /// Returns the definition of the current node if any.
    pub fn definition(&self) -> Option<&Definition> {
        self.defn.as_ref()
    }

    /// Sets the location of the definition for the current node.
    pub fn define(&mut self, defn: Definition) {
        self.defn = Some(defn);
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

/// Returns the type tag for the given node.
pub fn get_tag(n: NRef) -> Tag {
    n.syntax().core_ref().unwrap_tag()
}

/// Sets the type tag for the given node.
pub fn set_tag(n: NRef, t: Tag) {
    n.syntax().core_mut().set_tag(t)
}
