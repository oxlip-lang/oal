use indexmap::indexmap;
use oal_syntax::ast;
use openapiv3::{
    ArrayType, Info, MediaType, ObjectType, OpenAPI, Operation, Parameter, ParameterData,
    ParameterSchemaOrContent, PathItem, Paths, ReferenceOr, Response, Responses, Schema,
    SchemaData, SchemaKind, StringType, Type, VariantOrUnknownOrEmpty,
};

pub struct Builder {
    rels: Vec<ast::Rel>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder { rels: Vec::new() }
    }

    pub fn expose_all<'a, I: Iterator<Item = &'a ast::Rel>>(&mut self, rels: I) -> &Self {
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

    fn uri_pattern(&self, uri: &ast::Uri) -> String {
        uri.iter()
            .map(|s| match s {
                ast::UriSegment::Literal(l) => format!("/{}", l),
                ast::UriSegment::Variable(t) => format!("/{{{}}}", t.key),
            })
            .collect()
    }

    fn rel_pattern(&self, rel: &ast::Rel) -> String {
        match &rel.uri.inner {
            ast::Expr::Uri(uri) => self.uri_pattern(&uri),
            _ => panic!("expected uri type expression"),
        }
    }

    fn prim_type(&self, prim: &ast::Prim) -> Type {
        match prim {
            ast::Prim::Num => Type::Number(Default::default()),
            ast::Prim::Str => Type::String(Default::default()),
            ast::Prim::Bool => Type::Boolean {},
        }
    }

    fn prim_schema(&self, prim: &ast::Prim) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(self.prim_type(prim)),
        }
    }

    fn rel_schema(&self, rel: &ast::Rel) -> Schema {
        let uri = match &rel.uri.inner {
            ast::Expr::Uri(uri) => uri,
            _ => panic!("expected uri type"),
        };
        self.uri_schema(uri)
    }

    fn uri_schema(&self, uri: &ast::Uri) -> Schema {
        let pattern = if uri.is_empty() {
            None
        } else {
            Some(self.uri_pattern(uri).into())
        };
        Schema {
            schema_data: SchemaData {
                example: pattern,
                ..Default::default()
            },
            schema_kind: SchemaKind::Type(Type::String(StringType {
                format: VariantOrUnknownOrEmpty::Unknown("uri-reference".into()),
                ..Default::default()
            })),
        }
    }

    fn join_schema(&self, exprs: &Vec<ast::TypedExpr>) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::AllOf {
                all_of: exprs
                    .iter()
                    .map(|e| ReferenceOr::Item(self.expr_schema(&e.inner)))
                    .collect(),
            },
        }
    }

    fn block_type(&self, block: &ast::Block) -> Type {
        Type::Object(ObjectType {
            properties: block
                .iter()
                .map(|e| {
                    let ident = e.key.as_ref().into();
                    let expr = ReferenceOr::Item(self.expr_schema(&e.val.inner).into());
                    (ident, expr)
                })
                .collect(),
            ..Default::default()
        })
    }

    fn block_schema(&self, block: &ast::Block) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(self.block_type(block)),
        }
    }

    fn array_schema(&self, array: &ast::Array) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                items: Some(ReferenceOr::Item(
                    self.expr_schema(&array.item.inner).into(),
                )),
                min_items: None,
                max_items: None,
                unique_items: false,
            })),
        }
    }

    fn sum_schema(&self, exprs: &Vec<ast::TypedExpr>) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::OneOf {
                one_of: exprs
                    .iter()
                    .map(|e| ReferenceOr::Item(self.expr_schema(&e.inner)))
                    .collect(),
            },
        }
    }

    fn any_schema(&self, exprs: &Vec<ast::TypedExpr>) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::AnyOf {
                any_of: exprs
                    .iter()
                    .map(|e| ReferenceOr::Item(self.expr_schema(&e.inner)))
                    .collect(),
            },
        }
    }

    fn expr_schema(&self, e: &ast::Expr) -> Schema {
        match e {
            ast::Expr::Prim(prim) => self.prim_schema(prim),
            ast::Expr::Rel(rel) => self.rel_schema(rel),
            ast::Expr::Uri(uri) => self.uri_schema(uri),
            ast::Expr::Block(block) => self.block_schema(block),
            ast::Expr::Array(array) => self.array_schema(array),
            ast::Expr::Op(operation) => match operation.op {
                ast::Operator::Join => self.join_schema(&operation.exprs),
                ast::Operator::Sum => self.sum_schema(&operation.exprs),
                ast::Operator::Any => self.any_schema(&operation.exprs),
            },
            _ => panic!("unexpected type expression: {:?}", e),
        }
    }

    fn prop_param(&self, prop: &ast::Prop) -> Parameter {
        Parameter::Path {
            parameter_data: ParameterData {
                name: prop.key.as_ref().into(),
                description: None,
                required: true,
                deprecated: None,
                format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(
                    self.expr_schema(&prop.val.inner),
                )),
                example: None,
                examples: Default::default(),
                explode: None,
                extensions: Default::default(),
            },
            style: Default::default(),
        }
    }

    fn rel_params(&self, rel: &ast::Rel) -> Vec<Parameter> {
        match &rel.uri.inner {
            ast::Expr::Uri(uri) => uri
                .iter()
                .flat_map(|s| match s {
                    ast::UriSegment::Variable(p) => Some(self.prop_param(p)),
                    _ => None,
                })
                .collect(),
            _ => panic!("expected uri expression"),
        }
    }

    fn rel_path_item(&self, r: &ast::Rel) -> PathItem {
        let params = self
            .rel_params(r)
            .into_iter()
            .map(ReferenceOr::Item)
            .collect();
        let mut path_item = PathItem {
            parameters: params,
            ..Default::default()
        };
        let op = Operation {
            responses: Responses {
                default: Some(ReferenceOr::Item(Response {
                    content: indexmap! { self.media_type() => MediaType {
                        schema: Some(ReferenceOr::Item(self.expr_schema(&r.range.inner))),
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
                ast::Method::Post => path_item.post = Some(op.clone()),
                ast::Method::Patch => path_item.patch = Some(op.clone()),
                ast::Method::Delete => path_item.delete = Some(op.clone()),
                ast::Method::Options => path_item.options = Some(op.clone()),
                ast::Method::Head => path_item.head = Some(op.clone()),
            }
        }
        path_item
    }

    fn all_paths(&self) -> Paths {
        Paths {
            paths: self
                .rels
                .iter()
                .map(|r| {
                    (
                        self.rel_pattern(r),
                        ReferenceOr::Item(self.rel_path_item(r)),
                    )
                })
                .collect(),
            extensions: Default::default(),
        }
    }
}
