use crate::eval::{Expr, Object, Uri, UriSegment};
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
    let code = r#"
        # description: "some record"
        let r = {};
        res / ( put : <r> -> <r> );
    "#;
    let prg: Program = parse(code).expect("parsing failed");

    let s = evaluate(prg).expect("evaluation failed");

    assert_eq!(s.rels.len(), 1);

    let (i, p) = s.rels.iter().next().unwrap();

    assert_eq!(i, "/");
    assert_eq!(p.uri.spec.len(), 1);
    assert_eq!(*p.uri.spec.first().unwrap(), UriSegment::Literal("".into()));

    if let Some(x) = &p.xfers[oal_syntax::ast::Method::Put] {
        let d = x.domain.schema.as_ref().unwrap();
        assert_eq!(d.expr, Expr::Object(Object::default()));
        assert_eq!(d.desc, Some("some record".to_owned()));
        let r = x.range.schema.as_ref().unwrap();
        assert_eq!(r.expr, Expr::Object(Object::default()));
        assert_eq!(r.desc, Some("some record".to_owned()));
    } else {
        panic!("expected transfer on HTTP PUT");
    }
}

#[test]
fn evaluate_content() {
    let code = r#"
        let r = {};
        res / ( put : <r> -> <r> );
    "#;
    let prg: Program = parse(code).expect("parsing failed");

    let s = evaluate(prg).expect("evaluation failed");

    assert_eq!(s.rels.len(), 1);
}
