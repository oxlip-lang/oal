use crate::rewrite::module::External;
use oal_model::grammar::{NodeRef, SyntaxTree};
use oal_syntax::rewrite::parser::Gram;

#[derive(Clone, Default, Debug)]
pub struct Inner {
    defn: Option<External>,
}

impl Inner {
    pub fn define(&mut self, ext: External) {
        self.defn = Some(ext);
    }
}

pub type Tree = SyntaxTree<Inner, Gram>;

pub type NRef<'a> = NodeRef<'a, Inner, Gram>;
