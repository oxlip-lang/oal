use super::disjoin::{reduce, TagUnionFind};
use super::{Seq, Tag};
use oal_model::locator::Locator;

#[test]
fn disjoin() {
    let mut sets = TagUnionFind::new();

    let loc = Locator::try_from("file::///module.oal").unwrap();
    let mut seq = Seq::new(loc);

    let v0 = Tag::Var(seq.next());
    let v1 = Tag::Var(seq.next());
    let v2 = Tag::Var(seq.next());
    let v3 = Tag::Var(seq.next());

    sets.union(v0.clone(), Tag::Property(v2.clone().into()));
    sets.union(v1.clone(), v3.clone());
    sets.union(v3.clone(), v1.clone());
    sets.union(v1, v0);
    sets.union(v2, Tag::Number);

    let tag = reduce(&sets, &v3);

    assert_eq!(tag, Tag::Property(Tag::Number.into()));
}
