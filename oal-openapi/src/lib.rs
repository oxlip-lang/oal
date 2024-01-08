mod oas;

use crate::oas::into_box_ref;
use indexmap::{indexmap, IndexMap};
use oal_compiler::spec;
use oal_compiler::spec::SchemaExpr;
use oal_syntax::atom;
use openapiv3::*;
use std::iter::once;

pub struct Builder {
    spec: spec::Spec,
    base: Option<OpenAPI>,
}

type Headers = IndexMap<String, ReferenceOr<Header>>;
type Examples = IndexMap<String, ReferenceOr<Example>>;

impl From<Builder> for OpenAPI {
    fn from(b: Builder) -> Self {
        b.into_openapi()
    }
}

impl Builder {
    pub fn new(spec: spec::Spec) -> Builder {
        Builder { spec, base: None }
    }

    pub fn with_base(mut self, base: OpenAPI) -> Self {
        self.base = Some(base);
        self
    }

    pub fn into_openapi(self) -> OpenAPI {
        let paths = self.all_paths();
        let components = self.all_components();
        let mut definition = if let Some(base) = self.base {
            base
        } else {
            self.default_base()
        };
        definition.paths = paths;
        // Keep non-schema components
        definition
            .components
            .get_or_insert(Default::default())
            .schemas = components.schemas;
        definition
    }

    fn default_base(&self) -> OpenAPI {
        OpenAPI {
            openapi: "3.0.3".into(),
            info: Info {
                title: "OpenAPI definition".into(),
                version: "0.1.0".into(),
                ..Default::default()
            },
            servers: vec![Server {
                url: "/".to_owned(),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn media_type(&self) -> String {
        "application/json".to_owned()
    }

    fn uri_example_default(&self, uri: &spec::Uri) -> String {
        uri.pattern_with(|p| {
            let t = match p.schema.expr {
                SchemaExpr::Num(_) => "number",
                SchemaExpr::Str(_) => "string",
                SchemaExpr::Bool(_) => "boolean",
                SchemaExpr::Int(_) => "integer",
                _ => "unknown",
            };
            format!("_{}_{}_", p.name, t)
        })
    }

    fn number_schema(&self, p: &spec::PrimNumber) -> Schema {
        let example = p.example.map(Into::into);
        Schema {
            schema_data: SchemaData {
                example,
                ..Default::default()
            },
            schema_kind: SchemaKind::Type(Type::Number(NumberType {
                minimum: p.minimum,
                maximum: p.maximum,
                multiple_of: p.multiple_of,
                ..Default::default()
            })),
        }
    }

    fn string_schema(&self, p: &spec::PrimString) -> Schema {
        let example = p
            .example
            .clone()
            .or_else(|| p.enumeration.first().cloned())
            .map(Into::into);
        let format = match p.format {
            Some(ref f) => VariantOrUnknownOrEmpty::Unknown(f.clone()),
            None => VariantOrUnknownOrEmpty::Empty,
        };
        Schema {
            schema_data: SchemaData {
                example,
                ..Default::default()
            },
            schema_kind: SchemaKind::Type(Type::String(StringType {
                format,
                min_length: p.min_length,
                max_length: p.max_length,
                pattern: p.pattern.clone(),
                enumeration: p.enumeration.iter().map(|s| Some(s.clone())).collect(),
            })),
        }
    }

    fn boolean_schema(&self, _: &spec::PrimBoolean) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Boolean(BooleanType::default())),
        }
    }

    fn integer_schema(&self, p: &spec::PrimInteger) -> Schema {
        let example = p.example.map(Into::into);
        Schema {
            schema_data: SchemaData {
                example,
                ..Default::default()
            },
            schema_kind: SchemaKind::Type(Type::Integer(IntegerType {
                minimum: p.minimum,
                maximum: p.maximum,
                multiple_of: p.multiple_of,
                ..Default::default()
            })),
        }
    }

    fn rel_schema(&self, rel: &spec::Relation) -> Schema {
        self.uri_schema(&rel.uri)
    }

    fn uri_schema(&self, uri: &spec::Uri) -> Schema {
        let example = uri
            .example
            .clone()
            .or_else(|| {
                if uri.path.is_empty() {
                    None
                } else {
                    Some(self.uri_example_default(uri))
                }
            })
            .map(Into::into);
        Schema {
            schema_data: SchemaData {
                example,
                ..Default::default()
            },
            schema_kind: SchemaKind::Type(Type::String(StringType {
                format: VariantOrUnknownOrEmpty::Unknown("uri-reference".into()),
                ..Default::default()
            })),
        }
    }

    fn join_schema(&self, schemas: &[spec::Schema]) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::AllOf {
                all_of: schemas.iter().map(|s| self.schema(s)).collect(),
            },
        }
    }

    fn object_type(&self, obj: &spec::Object) -> Type {
        let properties = obj
            .props
            .iter()
            .map(|p| {
                let ident = p.name.as_ref().into();
                let expr = into_box_ref(self.schema(&p.schema));
                (ident, expr)
            })
            .collect();
        let required = obj
            .props
            .iter()
            .filter_map(|p| {
                if p.required.or(p.schema.required).unwrap_or(false) {
                    Some(p.name.as_ref().to_owned())
                } else {
                    None
                }
            })
            .collect();
        Type::Object(ObjectType {
            properties,
            required,
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
                items: Some(into_box_ref(self.schema(&array.item))),
                min_items: None,
                max_items: None,
                unique_items: false,
            })),
        }
    }

    fn sum_schema(&self, schemas: &[spec::Schema]) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::OneOf {
                one_of: schemas.iter().map(|s| self.schema(s)).collect(),
            },
        }
    }

    fn any_schema(&self, schemas: &[spec::Schema]) -> Schema {
        Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::AnyOf {
                any_of: schemas.iter().map(|s| self.schema(s)).collect(),
            },
        }
    }

    fn maybe_inline(&self, name: &atom::Ident) -> Option<&spec::Schema> {
        // Implicit and atomic references should be inlined.
        if name.is_reference() {
            return None;
        }
        let spec::Reference::Schema(s) = self.spec.refs.get(name).expect("reference should exist");
        match s.expr {
            spec::SchemaExpr::Num(_)
            | spec::SchemaExpr::Str(_)
            | spec::SchemaExpr::Bool(_)
            | spec::SchemaExpr::Int(_)
            | spec::SchemaExpr::Rel(_)
            | spec::SchemaExpr::Uri(_) => Some(s),
            _ => None,
        }
    }

    fn reference_schema(&self, name: &atom::Ident) -> ReferenceOr<Schema> {
        if let Some(s) = self.maybe_inline(name) {
            self.value_schema(s)
        } else {
            ReferenceOr::Reference {
                reference: format!("#/components/schemas/{}", name.untagged()),
            }
        }
    }

    fn value_schema(&self, s: &spec::Schema) -> ReferenceOr<Schema> {
        let mut sch = match &s.expr {
            spec::SchemaExpr::Num(p) => self.number_schema(p),
            spec::SchemaExpr::Str(p) => self.string_schema(p),
            spec::SchemaExpr::Bool(p) => self.boolean_schema(p),
            spec::SchemaExpr::Int(p) => self.integer_schema(p),
            spec::SchemaExpr::Rel(rel) => self.rel_schema(rel),
            spec::SchemaExpr::Uri(uri) => self.uri_schema(uri),
            spec::SchemaExpr::Object(obj) => self.object_schema(obj),
            spec::SchemaExpr::Array(array) => self.array_schema(array),
            spec::SchemaExpr::Op(operation) => match operation.op {
                atom::VariadicOperator::Join => self.join_schema(&operation.schemas),
                atom::VariadicOperator::Sum => self.sum_schema(&operation.schemas),
                atom::VariadicOperator::Any => self.any_schema(&operation.schemas),
                atom::VariadicOperator::Range => unreachable!(),
            },
            spec::SchemaExpr::Ref(_) => unreachable!(),
        };
        sch.schema_data.description = s.desc.clone();
        sch.schema_data.title = s.title.clone();
        ReferenceOr::Item(sch)
    }

    fn schema(&self, s: &spec::Schema) -> ReferenceOr<Schema> {
        if let spec::SchemaExpr::Ref(name) = &s.expr {
            self.reference_schema(name)
        } else {
            self.value_schema(s)
        }
    }

    fn prop_param_data(&self, prop: &spec::Property, required: bool) -> ParameterData {
        ParameterData {
            name: prop.name.as_ref().into(),
            description: prop.desc.clone(),
            required,
            deprecated: None,
            format: ParameterSchemaOrContent::Schema(self.schema(&prop.schema)),
            example: None,
            examples: Default::default(),
            explode: None,
            extensions: Default::default(),
        }
    }

    fn prop_path_param(&self, prop: &spec::Property) -> Parameter {
        Parameter::Path {
            parameter_data: self.prop_param_data(prop, true),
            style: Default::default(),
        }
    }

    fn prop_query_param(&self, prop: &spec::Property) -> Parameter {
        Parameter::Query {
            parameter_data: self.prop_param_data(prop, prop.required.unwrap_or(false)),
            allow_reserved: false,
            style: Default::default(),
            allow_empty_value: None,
        }
    }

    fn prop_header_param(&self, prop: &spec::Property) -> Parameter {
        Parameter::Header {
            parameter_data: self.prop_param_data(prop, prop.required.unwrap_or(false)),
            style: Default::default(),
        }
    }

    fn prop_header(&self, prop: &spec::Property) -> Header {
        Header {
            description: prop.desc.clone(),
            style: Default::default(),
            required: prop.required.unwrap_or(false),
            deprecated: None,
            format: ParameterSchemaOrContent::Schema(self.schema(&prop.schema)),
            example: None,
            examples: Default::default(),
            extensions: Default::default(),
        }
    }

    fn xfer_params(&self, xfer: &spec::Transfer) -> Vec<ReferenceOr<Parameter>> {
        let mut params = Vec::new();
        if let Some(o) = xfer.params.as_ref() {
            for p in o.props.iter() {
                params.push(ReferenceOr::Item(self.prop_query_param(p)));
            }
        }
        if let Some(o) = xfer.domain.headers.as_ref() {
            for p in o.props.iter() {
                params.push(ReferenceOr::Item(self.prop_header_param(p)));
            }
        }
        params
    }

    fn uri_params(&self, uri: &spec::Uri) -> Vec<ReferenceOr<Parameter>> {
        let mut params = Vec::new();
        for s in uri.path.iter() {
            if let spec::UriSegment::Variable(p) = s {
                params.push(ReferenceOr::Item(self.prop_path_param(p)));
            }
        }
        if let Some(o) = uri.params.as_ref() {
            for p in o.props.iter() {
                params.push(ReferenceOr::Item(self.prop_query_param(p)));
            }
        }
        params
    }

    fn domain_request(&self, domain: &spec::Content) -> Option<ReferenceOr<RequestBody>> {
        let media = domain.media.clone().unwrap_or_else(|| self.media_type());
        domain.schema.as_ref().map(|schema| {
            ReferenceOr::Item(RequestBody {
                content: indexmap! { media => MediaType {
                    schema: Some(self.schema(schema)),
                    examples: self.content_examples(domain),
                    ..Default::default()
                }},
                description: domain.desc.clone(),
                ..Default::default()
            })
        })
    }

    fn xfer_request(&self, xfer: &spec::Transfer) -> Option<ReferenceOr<RequestBody>> {
        self.domain_request(&xfer.domain)
    }

    fn http_status_code(&self, status: &atom::HttpStatus) -> StatusCode {
        match *status {
            atom::HttpStatus::Code(code) => StatusCode::Code(code.into()),
            atom::HttpStatus::Range(range) => StatusCode::Range(match range {
                atom::HttpStatusRange::Info => 1,
                atom::HttpStatusRange::Success => 2,
                atom::HttpStatusRange::Redirect => 3,
                atom::HttpStatusRange::ClientError => 4,
                atom::HttpStatusRange::ServerError => 5,
            }),
        }
    }

    fn content_headers(&self, content: &spec::Content) -> Headers {
        content.headers.as_ref().map_or_else(Headers::default, |h| {
            h.props
                .iter()
                .map(|p| {
                    (
                        p.name.as_ref().to_owned(),
                        ReferenceOr::Item(self.prop_header(p)),
                    )
                })
                .collect()
        })
    }

    fn content_examples(&self, content: &spec::Content) -> Examples {
        match content
            .examples
            .as_ref()
            .or_else(|| content.schema.as_ref().and_then(|s| s.examples.as_ref()))
        {
            None => Default::default(),
            Some(examples) => examples
                .iter()
                .map(|(name, url)| {
                    let example = Example {
                        external_value: Some(url.clone()),
                        ..Default::default()
                    };
                    (name.clone(), ReferenceOr::Item(example))
                })
                .collect(),
        }
    }

    fn xfer_responses(&self, xfer: &spec::Transfer) -> Responses {
        let mut default = None;
        let mut responses = IndexMap::new();

        for ((status, media), content) in xfer.ranges.iter() {
            let response = if let Some(s) = status {
                responses
                    .entry(self.http_status_code(s))
                    .or_insert(ReferenceOr::Item(Response::default()))
            } else {
                default.insert(ReferenceOr::Item(Response::default()))
            };
            if let ReferenceOr::Item(res) = response {
                if let Some(schema) = content.schema.as_ref() {
                    let media_type = media.clone().unwrap_or_else(|| self.media_type());
                    let media_schema = MediaType {
                        schema: Some(self.schema(schema)),
                        examples: self.content_examples(content),
                        ..Default::default()
                    };
                    res.content.insert(media_type, media_schema);
                }
                res.headers = self.content_headers(content);
                res.description = content.desc.clone().unwrap_or_else(|| "".to_owned());
            } else {
                unreachable!();
            }
        }

        Responses {
            default,
            responses,
            ..Default::default()
        }
    }

    fn method_label(&self, m: atom::Method) -> &str {
        match m {
            atom::Method::Get => "get",
            atom::Method::Put => "put",
            atom::Method::Post => "post",
            atom::Method::Patch => "patch",
            atom::Method::Delete => "delete",
            atom::Method::Options => "options",
            atom::Method::Head => "head",
        }
    }

    fn uri_segment_label(&self, s: &spec::UriSegment) -> String {
        match s {
            spec::UriSegment::Literal(l) => {
                let l = l.as_ref();
                if l.is_empty() {
                    "root".to_owned()
                } else {
                    l.to_lowercase()
                }
            }
            spec::UriSegment::Variable(t) => t.name.as_ref().to_lowercase(),
        }
    }

    fn xfer_id(
        &self,
        xfer: &spec::Transfer,
        method: atom::Method,
        uri: &spec::Uri,
    ) -> Option<String> {
        if xfer.id.is_some() {
            return xfer.id.clone();
        }
        let prefix = self.method_label(method).to_owned();
        let label = once(prefix)
            .chain(uri.path.iter().map(|s| self.uri_segment_label(s)))
            .collect::<Vec<_>>()
            .join("-");
        Some(label)
    }

    fn relation_path_item(&self, rel: &spec::Relation) -> PathItem {
        let mut path_item = PathItem {
            parameters: self.uri_params(&rel.uri),
            ..Default::default()
        };

        let xfers = rel
            .xfers
            .iter()
            .filter_map(|(m, x)| x.as_ref().map(|x| (m, x)));

        for (method, xfer) in xfers {
            let operation_id = self.xfer_id(xfer, method, &rel.uri);
            let summary = xfer
                .summary
                .clone()
                .or_else(|| xfer.desc.clone())
                .or_else(|| operation_id.clone());
            let description = xfer.desc.clone();

            let op = Operation {
                summary,
                description,
                operation_id,
                parameters: self.xfer_params(xfer),
                request_body: self.xfer_request(xfer),
                responses: self.xfer_responses(xfer),
                tags: xfer.tags.clone(),
                ..Default::default()
            };

            match method {
                atom::Method::Get => path_item.get = Some(op),
                atom::Method::Put => path_item.put = Some(op),
                atom::Method::Post => path_item.post = Some(op),
                atom::Method::Patch => path_item.patch = Some(op),
                atom::Method::Delete => path_item.delete = Some(op),
                atom::Method::Options => path_item.options = Some(op),
                atom::Method::Head => path_item.head = Some(op),
            }
        }

        path_item
    }

    fn all_paths(&self) -> Paths {
        let paths = self
            .spec
            .rels
            .iter()
            .map(|rel| {
                (
                    rel.uri.pattern(),
                    ReferenceOr::Item(self.relation_path_item(rel)),
                )
            })
            .collect();
        Paths {
            paths,
            extensions: Default::default(),
        }
    }

    fn all_components(&self) -> Components {
        let mut schemas = IndexMap::new();
        for (name, spec::Reference::Schema(s)) in self.spec.refs.iter() {
            // Only keep components that couldn't be inlined.
            if self.maybe_inline(name).is_none() {
                schemas.insert(name.untagged(), self.schema(s));
            }
        }
        Components {
            schemas,
            ..Default::default()
        }
    }
}
