use super::tag::{FuncTag, Tag};
use indexmap::IndexSet;

/// An implementation of a union-find/disjoint-set data structure
/// for reducing equivalences between [`Tag`] values.
#[derive(Debug)]
pub struct UnionFind {
    tags: IndexSet<Tag>,
    parents: Vec<usize>,
}

impl UnionFind {
    /// Creates a new set of disjoint sets.
    pub fn new() -> Self {
        UnionFind {
            tags: IndexSet::new(),
            parents: Vec::new(),
        }
    }

    /// Inserts a tag as a new disjoint set.
    fn insert(&mut self, tag: Tag) -> usize {
        let (index, _) = self.tags.insert_full(tag);
        if index == self.parents.len() {
            self.parents.push(index);
        }
        assert!(index < self.parents.len());
        index
    }

    /// Reduces a tag reference by walking the parent path.
    /// 
    /// Flattens the path on the way.
    fn reduce_mut(&mut self, v: usize) -> usize {
        let mut w = v;
        loop {
            let parent = self.parents[w];
            if parent == w {
                self.parents[v] = w;
                break w;
            }
            w = parent;
        }
    }

    /// Reduces a tag reference by walking the parent path.
    fn reduce(&self, mut v: usize) -> usize {
        loop {
            let parent = self.parents[v];
            if parent == v {
                break v;
            }
            v = parent;
        }
    }

    /// Joins the classes of equivalence corresponding to the `left` and `right` tags.
    /// 
    /// The representative of the `right` class always takes over as representative for the `left` class.
    pub fn union(&mut self, left: Tag, right: Tag) {
        let v = self.insert(left);
        let w = self.insert(right);
        let vrep = self.reduce_mut(v);
        let wrep = self.reduce_mut(w);
        self.parents[vrep] = wrep;
    }

    #[allow(dead_code)]
    pub fn find_mut<'a>(&'a mut self, tag: &Tag) -> Option<(&'a Tag, bool)> {
        if let Some((v, _)) = self.tags.get_full(tag) {
            let vrep = self.reduce_mut(v);
            Some((self.tags.get_index(vrep).unwrap(), vrep != v))
        } else {
            None
        }
    }

    /// Finds the representative of the class of equivalence given by `tag`.
    /// 
    /// If a representative is known, also returns whether a reduction happened.
    pub fn find<'a>(&'a self, tag: &Tag) -> Option<(&'a Tag, bool)> {
        if let Some((v, _)) = self.tags.get_full(tag) {
            let vrep = self.reduce(v);
            Some((self.tags.get_index(vrep).unwrap(), vrep != v))
        } else {
            None
        }
    }
}

/// Reduces a [`Tag`] according to the `sets` of classes of equivalence.
pub fn reduce(sets: &UnionFind, tag: &Tag) -> Tag {
    match tag {
        Tag::Var(_) => sets
            .find(tag)
            .and_then(|(t, reduced)| reduced.then(|| reduce(sets, t)))
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