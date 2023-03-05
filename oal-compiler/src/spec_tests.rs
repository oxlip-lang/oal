use crate::spec::{Object, PrimNumber, Property, Schema, SchemaExpr, Uri, UriSegment};

#[test]
fn uri_pattern() {
    let cases = [
        (
            Uri {
                path: vec![],
                params: None,
                example: None,
            },
            "",
        ),
        (
            Uri {
                path: vec![UriSegment::Literal("".into())],
                params: None,
                example: None,
            },
            "/",
        ),
        (
            Uri {
                path: vec![
                    UriSegment::Literal("a".into()),
                    UriSegment::Variable(
                        Property {
                            name: "b".into(),
                            schema: Schema {
                                expr: SchemaExpr::Int(Default::default()),
                                desc: None,
                                title: None,
                                required: None,
                                examples: None,
                            },
                            desc: None,
                            required: None,
                        }
                        .into(),
                    ),
                    UriSegment::Literal("c".into()),
                ],
                params: None,
                example: None,
            },
            "/a/{b}/c",
        ),
    ];

    for c in cases {
        assert_eq!(c.0.pattern(), c.1);
    }
}

fn make_param(name: &str) -> Object {
    Object {
        props: vec![Property {
            name: name.into(),
            schema: Schema {
                expr: SchemaExpr::Num(PrimNumber::default()),
                desc: None,
                title: None,
                required: None,
                examples: None,
            },
            desc: None,
            required: None,
        }],
    }
}

#[test]
fn uri_append() {
    for (mut left, right, exp) in [
        (
            Uri {
                path: vec![UriSegment::Literal("a".into())],
                params: Some(make_param("a")),
                example: Some("a".into()),
            },
            Uri {
                path: vec![UriSegment::Literal("b".into())],
                params: Some(make_param("b")),
                example: Some("b".into()),
            },
            Uri {
                path: vec![
                    UriSegment::Literal("a".into()),
                    UriSegment::Literal("b".into()),
                ],
                params: Some(make_param("b")),
                example: None,
            },
        ),
        (
            Uri {
                path: vec![
                    UriSegment::Literal("a".into()),
                    UriSegment::Literal("".into()),
                ],
                params: None,
                example: None,
            },
            Uri {
                path: vec![UriSegment::Literal("b".into())],
                params: None,
                example: None,
            },
            Uri {
                path: vec![
                    UriSegment::Literal("a".into()),
                    UriSegment::Literal("b".into()),
                ],
                params: None,
                example: None,
            },
        ),
    ] {
        left.append(right);
        assert_eq!(left, exp);
    }
}
