use crate::evaluate;
use oal_syntax::ast::{Block, Method, UriSegment};
use oal_syntax::parse;

#[test]
fn evaluate_simple() {
    let doc = parse("res /:put:{} -> {};".to_owned()).expect("parsing failed");

    let s = evaluate(doc).expect("evaluation failed");

    assert_eq!(s.paths.len(), 1);

    let (i, p) = s.paths.iter().next().unwrap();

    assert_eq!(i, "/");
    assert_eq!(p.uri.spec.len(), 1);
    assert_eq!(*p.uri.spec.first().unwrap(), UriSegment::default());

    assert_eq!(p.ops.len(), 1);

    let (m, o) = p.ops.iter().next().unwrap();

    assert_eq!(*m, Method::Put);
    assert_eq!(o.domain, Some(Block::default()));
    assert_eq!(o.range, Block::default());
}
