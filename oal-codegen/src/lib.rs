use indexmap::indexmap;
use oal_syntax::ast;
use oal_syntax::ast::TypeExpr;
use openapiv3::{
    Info, MediaType, OpenAPI, Operation, PathItem, Paths, ReferenceOr, Response, Responses, Schema,
    SchemaKind, StringType, Type, VariantOrUnknownOrEmpty,
};

pub struct Builder {
    rels: Vec<ast::TypeRel>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder { rels: Vec::new() }
    }

    pub fn expose_all<'a, I: Iterator<Item = &'a ast::TypeRel>>(&mut self, rels: I) -> &Self {
        self.rels = rels.cloned().collect();
        self
    }

    pub fn open_api(&self) -> OpenAPI {
        OpenAPI {
            openapi: "3.0.1".into(),
            info: Info {
                title: "Test OpenAPI specification".into(),
                version: "0.1.0".into(),
                ..Default::default()
            },
            paths: self.all_paths(),
            ..Default::default()
        }
    }

    fn media_type(&self) -> String {
        "application/json".into()
    }

    fn path_uri(&self, rel: &ast::TypeRel) -> String {
        match rel.uri.as_ref() {
            ast::TypeExpr::Uri(uri) => uri
                .into_iter()
                .map(|s| match s {
                    ast::UriSegment::Literal(l) => format!("/{}", l),
                    ast::UriSegment::Template(t) => format!("/{{{}}}", t.ident),
                })
                .collect(),
            _ => panic!("expected uri type expression"),
        }
    }

    fn prim_type(&self, prim: &ast::TypePrim) -> Type {
        match prim {
            ast::TypePrim::Num => Type::Number(Default::default()),
            ast::TypePrim::Str => Type::String(Default::default()),
            ast::TypePrim::Bool => Type::Boolean {},
        }
    }

    fn prim_schema(&self, prim: &ast::TypePrim) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(self.prim_type(prim)),
        }
    }

    fn rel_schema(&self, rel: &ast::TypeRel) -> Schema {
        let uri = match rel.uri.as_ref() {
            TypeExpr::Uri(uri) => uri,
            _ => panic!("expected uri type"),
        };
        self.uri_schema(uri)
    }

    fn uri_schema(&self, _uri: &ast::TypeUri) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::String(StringType {
                format: VariantOrUnknownOrEmpty::Unknown("uri-reference".into()),
                ..Default::default()
            })),
        }
    }

    fn default_schema(&self) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Any(Default::default()),
        }
    }

    fn expr_schema(&self, e: &ast::TypeExpr) -> Schema {
        match e {
            ast::TypeExpr::Prim(prim) => self.prim_schema(prim),
            ast::TypeExpr::Rel(rel) => self.rel_schema(rel),
            ast::TypeExpr::Uri(uri) => self.uri_schema(uri),
            ast::TypeExpr::Join(_) => self.default_schema(),
            ast::TypeExpr::Block(_) => self.default_schema(),
            ast::TypeExpr::Sum(_) => self.default_schema(),
            _ => panic!("unexpected type expression"),
        }
    }

    fn path_item(&self, r: &ast::TypeRel) -> PathItem {
        let mut path_item = PathItem {
            ..Default::default()
        };
        let op = Operation {
            responses: Responses {
                default: Some(ReferenceOr::Item(Response {
                    content: indexmap! { self.media_type() => MediaType {
                        schema: Some(ReferenceOr::Item(self.expr_schema(r.range.as_ref()))),
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
        path_item
    }

    fn all_paths(&self) -> Paths {
        Paths {
            paths: self
                .rels
                .iter()
                .map(|r| (self.path_uri(r), ReferenceOr::Item(self.path_item(r))))
                .collect(),
            extensions: Default::default(),
        }
    }
}
