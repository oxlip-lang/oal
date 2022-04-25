use crate::eval::{Object, Schema, Uri, UriSegment};
use crate::{evaluate, Program};
use oal_syntax::parse;

#[test]
fn uri_pattern() {
    let uri = Uri {
        spec: vec![UriSegment::Literal("".into())],
    };

    assert_eq!(uri.pattern(), "/");
}

#[test]
fn evaluate_simple() {
    let prg: Program = parse("res / ( put : {} -> {} );".to_owned()).expect("parsing failed");

    let s = evaluate(prg).expect("evaluation failed");

    assert_eq!(s.rels.len(), 1);

    let (i, p) = s.rels.iter().next().unwrap();

    assert_eq!(i, "/");
    assert_eq!(p.uri.spec.len(), 1);
    assert_eq!(*p.uri.spec.first().unwrap(), UriSegment::Literal("".into()));

    if let Some(x) = &p.xfers[oal_syntax::ast::Method::Put] {
        if let Some(d) = &x.domain {
            assert_eq!(*d.as_ref(), Schema::Object(Object::default()));
        } else {
            panic!("expected domain");
        }
        assert_eq!(*x.range, Schema::Object(Object::default()));
    } else {
        panic!("expected transfer on HTTP PUT");
    }
}
