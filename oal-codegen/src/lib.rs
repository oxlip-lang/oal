use indexmap::indexmap;
use oal_syntax::ast;
use oal_syntax::ast::UriSegment;
use openapiv3::{
    Info, MediaType, OpenAPI, Operation, PathItem, Paths, ReferenceOr, Response, Responses, Schema,
    SchemaData, SchemaKind, Type,
};

fn format_uri(uri: &ast::TypeUri) -> String {
    uri.into_iter()
        .map(|s| match s {
            UriSegment::Literal(l) => format!("/{}", l),
            UriSegment::Template(t) => format!("/{{{}}}", t.ident),
        })
        .collect()
}

fn format_schema(_e: &ast::TypeExpr) -> Schema {
    Schema {
        schema_data: SchemaData {
            ..Default::default()
        },
        schema_kind: SchemaKind::Type(Type::Boolean {}),
    }
}

pub struct Api {
    rels: Vec<ast::TypeRel>,
}

impl Api {
    pub fn new() -> Api {
        Api { rels: Vec::new() }
    }

    pub fn expose_all<'a, I: Iterator<Item = &'a ast::TypeRel>>(&mut self, rels: I) -> &Self {
        self.rels = rels.cloned().collect();
        self
    }

    pub fn render(&self) -> OpenAPI {
        OpenAPI {
            openapi: "3.0.1".into(),
            info: Info {
                title: "Test OpenAPI specification".into(),
                version: "0.1.0".into(),
                ..Default::default()
            },
            paths: self.paths(),
            ..Default::default()
        }
    }

    fn media_type(&self) -> String {
        "application/json".into()
    }

    fn paths(&self) -> Paths {
        Paths {
            paths: self
                .rels
                .iter()
                .map(|r| {
                    let uri = match r.uri.as_ref() {
                        ast::TypeExpr::Uri(uri) => uri,
                        _ => panic!("expected uri type expression"),
                    };
                    let mut path_item = PathItem {
                        ..Default::default()
                    };
                    let schema = format_schema(r.range.as_ref());
                    let op = Operation {
                        responses: Responses {
                            default: Some(ReferenceOr::Item(Response {
                                content: indexmap! { self.media_type() => MediaType {
                                    schema: Some(ReferenceOr::Item(schema)),
                                    ..Default::default()
                                }},
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    for m in &r.methods {
                        match m {
                            ast::Method::Get => path_item.get = Some(op.clone()),
                            ast::Method::Put => path_item.put = Some(op.clone()),
                        }
                    }
                    (format_uri(uri), ReferenceOr::Item(path_item))
                })
                .collect(),
            extensions: Default::default(),
        }
    }
}
