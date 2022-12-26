use super::tag::{FuncTag, Tag};
use std::collections::HashMap;

/// A naive implementation of a union-find/disjoint-set data structure
/// for storing equivalences between Tag values
/// and substituting a representative Tag from each equivalence class.
// TODO: replace with petgraph::UnionFind
#[derive(Debug, Default)]
pub struct Set(HashMap<usize, Tag>);

impl Set {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn extend(&mut self, v: usize, t: Tag) {
        self.0.insert(v, t);
    }

    pub fn substitute(&self, tag: &Tag) -> Tag {
        match tag {
            Tag::Var(v) => {
                if let Some(t) = self.0.get(v) {
                    self.substitute(t)
                } else {
                    tag.clone()
                }
            }
            Tag::Func(FuncTag { bindings, range }) => {
                let bindings = bindings.iter().map(|b| self.substitute(b)).collect();
                let range = self.substitute(range).into();
                Tag::Func(FuncTag { bindings, range })
            }
            Tag::Property(t) => Tag::Property(self.substitute(t.as_ref()).into()),
            _ => tag.clone(),
        }
    }
}
