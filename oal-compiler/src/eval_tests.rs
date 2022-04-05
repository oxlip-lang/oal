use crate::eval::{Object, Schema, Uri, UriSegment};
use crate::evaluate;
use oal_syntax::{ast, parse};

#[test]
fn uri_pattern() {
    let uri = Uri {
        spec: vec![UriSegment::Literal("".into())],
    };

    assert_eq!(uri.pattern(), "/");
}

#[test]
fn evaluate_simple() {
    let doc = parse("res /:put:{} -> {};".to_owned()).expect("parsing failed");

    let s = evaluate(doc).expect("evaluation failed");

    assert_eq!(s.rels.len(), 1);

    let (i, p) = s.rels.iter().next().unwrap();

    assert_eq!(i, "/");
    assert_eq!(p.uri.spec.len(), 1);
    assert_eq!(*p.uri.spec.first().unwrap(), UriSegment::Literal("".into()));

    assert_eq!(p.ops.len(), 1);

    let (m, o) = p.ops.iter().next().unwrap();

    assert_eq!(*m, ast::Method::Put);
    assert_eq!(o.domain, Some(Schema::Object(Object::default())));
    assert_eq!(o.range, Schema::Object(Object::default()));
}
