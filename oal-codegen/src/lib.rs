use indexmap::indexmap;
use oal_compiler::spec;
use oal_syntax::ast;
use openapiv3::{
    ArrayType, Info, MediaType, ObjectType, OpenAPI, Operation, Parameter, ParameterData,
    ParameterSchemaOrContent, PathItem, Paths, ReferenceOr, RequestBody, Response, Responses,
    Schema, SchemaData, SchemaKind, Server, StringType, Type, VariantOrUnknownOrEmpty,
};

pub struct Builder {
    spec: spec::Spec,
}

impl Builder {
    pub fn new(s: spec::Spec) -> Builder {
        Builder { spec: s }
    }

    pub fn open_api(&self) -> OpenAPI {
        OpenAPI {
            openapi: "3.0.1".into(),
            info: Info {
                title: "Test OpenAPI specification".into(),
                version: "0.1.0".into(),
                ..Default::default()
            },
            servers: vec![Server {
                url: "/".to_owned(),
                ..Default::default()
            }],
            paths: self.all_paths(),
            ..Default::default()
        }
    }

    fn media_type(&self) -> String {
        "application/json".into()
    }

    fn uri_pattern(&self, uri: &spec::Uri) -> String {
        uri.pattern()
    }

    fn prim_type(&self, prim: &ast::Primitive) -> Type {
        match prim {
            ast::Primitive::Num => Type::Number(Default::default()),
            ast::Primitive::Str => Type::String(Default::default()),
            ast::Primitive::Bool => Type::Boolean {},
        }
    }

    fn prim_schema(&self, prim: &ast::Primitive) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(self.prim_type(prim)),
        }
    }

    fn rel_schema(&self, rel: &spec::Relation) -> Schema {
        self.uri_schema(&rel.uri)
    }

    fn uri_schema(&self, uri: &spec::Uri) -> Schema {
        let pattern = if uri.spec.is_empty() {
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

    fn join_schema(&self, schemas: &Vec<spec::Schema>) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::AllOf {
                all_of: schemas
                    .iter()
                    .map(|s| ReferenceOr::Item(self.schema(s)))
                    .collect(),
            },
        }
    }

    fn object_type(&self, obj: &spec::Object) -> Type {
        Type::Object(ObjectType {
            properties: obj
                .props
                .iter()
                .map(|p| {
                    let ident = p.name.as_ref().into();
                    let expr = ReferenceOr::Item(self.schema(&p.schema).into());
                    (ident, expr)
                })
                .collect(),
            ..Default::default()
        })
    }

    fn object_schema(&self, obj: &spec::Object) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(self.object_type(obj)),
        }
    }

    fn array_schema(&self, array: &spec::Array) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                items: Some(ReferenceOr::Item(self.schema(array.item.as_ref()).into())),
                min_items: None,
                max_items: None,
                unique_items: false,
            })),
        }
    }

    fn sum_schema(&self, schemas: &Vec<spec::Schema>) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::OneOf {
                one_of: schemas
                    .iter()
                    .map(|s| ReferenceOr::Item(self.schema(s)))
                    .collect(),
            },
        }
    }

    fn any_schema(&self, schemas: &Vec<spec::Schema>) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::AnyOf {
                any_of: schemas
                    .iter()
                    .map(|s| ReferenceOr::Item(self.schema(s)))
                    .collect(),
            },
        }
    }

    fn schema(&self, s: &spec::Schema) -> Schema {
        let mut sch = match &s.expr {
            spec::Expr::Prim(prim) => self.prim_schema(prim),
            spec::Expr::Rel(rel) => self.rel_schema(rel),
            spec::Expr::Uri(uri) => self.uri_schema(uri),
            spec::Expr::Object(obj) => self.object_schema(obj),
            spec::Expr::Array(array) => self.array_schema(array),
            spec::Expr::Op(operation) => match operation.op {
                ast::Operator::Join => self.join_schema(&operation.schemas),
                ast::Operator::Sum => self.sum_schema(&operation.schemas),
                ast::Operator::Any => self.any_schema(&operation.schemas),
            },
        };
        sch.schema_data.description = s.desc.clone();
        sch.schema_data.title = s.title.clone();
        sch
    }

    fn prop_param(&self, prop: &spec::Prop) -> Parameter {
        Parameter::Path {
            parameter_data: ParameterData {
                name: prop.name.as_ref().into(),
                description: None,
                required: true,
                deprecated: None,
                format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(
                    self.schema(&prop.schema),
                )),
                example: None,
                examples: Default::default(),
                explode: None,
                extensions: Default::default(),
            },
            style: Default::default(),
        }
    }

    fn uri_params(&self, uri: &spec::Uri) -> Vec<Parameter> {
        uri.spec
            .iter()
            .flat_map(|s| match s {
                spec::UriSegment::Variable(p) => Some(self.prop_param(p)),
                _ => None,
            })
            .collect()
    }

    fn transfer_request(&self, xfer: &spec::Transfer) -> Option<ReferenceOr<RequestBody>> {
        xfer.domain.schema.as_ref().map(|schema| {
            ReferenceOr::Item(RequestBody {
                content: indexmap! { self.media_type() => MediaType {
                    schema: Some(ReferenceOr::Item(self.schema(schema))),
                    ..Default::default()
                }},
                description: xfer.domain.desc.clone(),
                ..Default::default()
            })
        })
    }

    fn transfer_response(&self, xfer: &spec::Transfer) -> Option<ReferenceOr<Response>> {
        xfer.range.schema.as_ref().map(|schema| {
            ReferenceOr::Item(Response {
                content: indexmap! { self.media_type() => MediaType {
                    schema: Some(ReferenceOr::Item(self.schema(schema))),
                    ..Default::default()
                }},
                description: xfer.range.desc.clone().unwrap_or("".to_owned()),
                ..Default::default()
            })
        })
    }

    fn relation_path_item(&self, rel: &spec::Relation) -> PathItem {
        let parameters = self
            .uri_params(&rel.uri)
            .into_iter()
            .map(ReferenceOr::Item)
            .collect();

        let mut path_item = PathItem {
            parameters,
            ..Default::default()
        };

        let xfers = rel
            .xfers
            .iter()
            .filter_map(|(m, x)| x.as_ref().map(|x| (m, x)));

        for (method, xfer) in xfers {
            let op = Operation {
                summary: xfer.summary.clone(),
                request_body: self.transfer_request(xfer),
                responses: Responses {
                    default: self.transfer_response(xfer),
                    ..Default::default()
                },
                ..Default::default()
            };

            match method {
                ast::Method::Get => path_item.get = Some(op),
                ast::Method::Put => path_item.put = Some(op),
                ast::Method::Post => path_item.post = Some(op),
                ast::Method::Patch => path_item.patch = Some(op),
                ast::Method::Delete => path_item.delete = Some(op),
                ast::Method::Options => path_item.options = Some(op),
                ast::Method::Head => path_item.head = Some(op),
            }
        }

        path_item
    }

    fn all_paths(&self) -> Paths {
        Paths {
            paths: self
                .spec
                .rels
                .iter()
                .map(|(pattern, rel)| {
                    (
                        pattern.clone(),
                        ReferenceOr::Item(self.relation_path_item(rel)),
                    )
                })
                .collect(),
            extensions: Default::default(),
        }
    }
}
