use crate::spec::{Property, Schema, SchemaExpr, Uri, UriSegment};

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
