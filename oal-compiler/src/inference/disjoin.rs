use super::tag::{FuncTag, Tag, TagId};
use indexmap::IndexSet;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug)]
pub struct TagUnionFind {
    tags: IndexSet<Tag>,
    vars: Vec<usize>,
}

#[allow(dead_code)]
impl TagUnionFind {
    pub fn new() -> Self {
        TagUnionFind {
            tags: IndexSet::new(),
            vars: Vec::new(),
        }
    }

    fn insert(&mut self, tag: Tag) -> usize {
        let (index, _) = self.tags.insert_full(tag);
        if index == self.vars.len() {
            self.vars.push(index);
        }
        assert!(index < self.vars.len());
        index
    }

    fn reduce(&mut self, v: usize) -> usize {
        let mut w = v;
        loop {
            let parent = self.vars[w];
            if parent == w {
                self.vars[v] = w;
                break w;
            }
            w = parent;
        }
    }

    pub fn union(&mut self, i: Tag, j: Tag) {
        let v = self.insert(i);
        let w = self.insert(j);
        let vrep = self.reduce(v);
        let wrep = self.reduce(w);
        self.vars[vrep] = wrep;
    }

    pub fn find(&self, tag: &Tag) -> Option<&Tag> {
        if let Some((v, _)) = self.tags.get_full(tag) {
            let vrep = self.vars[v];
            Some(self.tags.get_index(vrep).unwrap())
        } else {
            None
        }
    }
}

#[allow(dead_code)]
pub fn reduce(sets: &TagUnionFind, tag: &Tag) -> Tag {
    match tag {
        Tag::Var(_) => sets
            .find(tag)
            .map(|t| reduce(sets, t))
            .unwrap_or_else(|| tag.clone()),
        Tag::Func(FuncTag { bindings, range }) => {
            let bindings = bindings.iter().map(|b| reduce(sets, b)).collect();
            let range = reduce(sets, range).into();
            Tag::Func(FuncTag { bindings, range })
        }
        Tag::Property(t) => Tag::Property(reduce(sets, t.as_ref()).into()),
        _ => tag.clone(),
    }
}

/// A naive implementation of a union-find/disjoint-set data structure
/// for storing equivalences between Tag values
/// and substituting a representative Tag from each equivalence class.
// TODO: deprecate in favor of TagUnionFind
#[derive(Debug, Default)]
pub struct Set(HashMap<TagId, Tag>);

impl Set {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn extend(&mut self, v: TagId, t: Tag) {
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
