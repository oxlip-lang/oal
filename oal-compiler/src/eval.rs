use crate::annotation::Annotation;
use crate::errors::Result;
use crate::module::ModuleSet;
use crate::spec::{
    Array, Content, Object, PrimBoolean, PrimInteger, PrimNumber, PrimString, Property, Ranges,
    Reference, Relation, Schema, SchemaExpr, Spec, Transfer, Transfers, Uri, UriSegment,
    VariadicOp,
};
use crate::tree::{definition, Core, NRef};
use enum_map::EnumMap;
use indexmap::IndexMap;
use oal_model::locator::Locator;
use oal_syntax::atom;
use oal_syntax::lexer as lex;
use oal_syntax::parser as syn;
use std::collections::HashMap;
use std::rc::Rc;

// AnnRef is the type of references to annotations.
type AnnRef = Rc<Annotation>;

// Value is the type of evaluation results.
type Value<'a> = (Expr<'a>, AnnRef);

// Expr is the type of evaluated expressions.
#[derive(Clone, Debug)]
enum Expr<'a> {
    Spec(Box<Spec>),
    Uri(Box<Uri>),
    Relation(Box<Relation>),
    Transfer(Box<Transfer>),
    Content(Box<Content>),
    Object(Box<Object>),
    Ranges(Box<Ranges>),
    Property(Box<Property>),
    PrimInteger(Box<PrimInteger>),
    PrimNumber(Box<PrimNumber>),
    PrimString(Box<PrimString>),
    PrimBoolean(Box<PrimBoolean>),
    VariadicOp(Box<VariadicOp>),
    Reference(atom::Ident, Box<Value<'a>>),
    Array(Box<Array>),
    String(String),
    Number(u64),
    HttpStatus(atom::HttpStatus),
    Lambda(syn::Declaration<'a, Core>),
}

impl<'a> Expr<'a> {
    fn is_schema_like(&self) -> bool {
        matches!(
            self,
            Expr::Object(_)
                | Expr::PrimInteger(_)
                | Expr::PrimNumber(_)
                | Expr::PrimString(_)
                | Expr::PrimBoolean(_)
                | Expr::Array(_)
                | Expr::Uri(_)
                | Expr::VariadicOp(_)
                | Expr::Reference(_, _)
                | Expr::Relation(_)
        )
    }

    fn is_content_like(&self) -> bool {
        matches!(self, Expr::Content(_)) || self.is_schema_like()
    }

    fn is_uri_like(&self) -> bool {
        matches!(self, Expr::Uri(_) | Expr::Relation(_))
    }
}

struct Context<'a> {
    mods: &'a ModuleSet,
    refs: IndexMap<atom::Ident, Value<'a>>,
    scopes: Vec<HashMap<atom::Ident, syn::Terminal<'a, Core>>>,
}

impl<'a> Context<'a> {
    fn new(mods: &'a ModuleSet) -> Self {
        Context {
            mods,
            refs: IndexMap::new(),
            scopes: Vec::new(),
        }
    }
}

fn compose_annotations<'a, I>(iter: I) -> Result<Annotation>
where
    I: Iterator<Item = &'a str>,
{
    let mut ann = Annotation::default();
    for text in iter {
        let other = Annotation::try_from(text)?;
        ann.extend(other);
    }
    Ok(ann)
}

fn cast_schema(from: (Expr, AnnRef)) -> Schema {
    let ann = from.1;
    let desc = ann.get_string("description");
    let title = ann.get_string("title");
    let required = ann.get_bool("required");
    let examples = ann.get_props("examples");

    let expr = match from.0 {
        Expr::Object(o) => SchemaExpr::Object(*o),
        Expr::PrimInteger(i) => SchemaExpr::Int(*i),
        Expr::PrimNumber(p) => SchemaExpr::Num(*p),
        Expr::PrimString(s) => SchemaExpr::Str(*s),
        Expr::PrimBoolean(b) => SchemaExpr::Bool(*b),
        Expr::Array(a) => SchemaExpr::Array(a),
        Expr::Uri(u) => SchemaExpr::Uri(*u),
        Expr::VariadicOp(o) => SchemaExpr::Op(*o),
        Expr::Reference(r, _) => SchemaExpr::Ref(r),
        Expr::Relation(r) => SchemaExpr::Rel(r),
        e => panic!("not a schema: {:?}", e),
    };

    Schema {
        expr,
        desc,
        title,
        required,
        examples,
    }
}

fn cast_content(from: (Expr, AnnRef)) -> Content {
    if let Expr::Content(c) = from.0 {
        *c
    } else if from.0.is_schema_like() {
        Content::from(cast_schema(from))
    } else if let Expr::Reference(_, v) = from.0 {
        cast_content(*v)
    } else {
        panic!("not a content: {:?}", from.0)
    }
}

fn cast_ranges(from: (Expr, AnnRef)) -> Ranges {
    if let Expr::Ranges(r) = from.0 {
        *r
    } else if from.0.is_content_like() {
        let c = cast_content(from);
        Ranges::from([((c.status, c.media.clone()), c)])
    } else if let Expr::Reference(_, v) = from.0 {
        cast_ranges(*v)
    } else {
        panic!("not ranges: {:?}", from.0)
    }
}

fn cast_string(from: (Expr, AnnRef)) -> String {
    match from.0 {
        Expr::String(s) => s,
        Expr::Reference(_, v) => cast_string(*v),
        e => panic!("not a string: {:?}", e),
    }
}

fn cast_property(from: (Expr, AnnRef)) -> Property {
    match from.0 {
        Expr::Property(p) => *p,
        Expr::Reference(_, v) => cast_property(*v),
        e => panic!("not a property: {:?}", e),
    }
}

fn cast_http_status(from: (Expr, AnnRef)) -> Result<atom::HttpStatus> {
    match from.0 {
        Expr::HttpStatus(s) => Ok(s),
        Expr::Number(n) => {
            let s = atom::HttpStatus::try_from(n)?;
            Ok(s)
        }
        Expr::Reference(_, v) => cast_http_status(*v),
        e => panic!("not an HTTP status: {:?}", e),
    }
}

fn cast_object(from: (Expr, AnnRef)) -> Object {
    match from.0 {
        Expr::Object(o) => *o,
        Expr::Reference(_, v) => cast_object(*v),
        e => panic!("not an object: {:?}", e),
    }
}

fn cast_transfer(from: (Expr, AnnRef)) -> Transfer {
    match from.0 {
        Expr::Transfer(x) => *x,
        Expr::Reference(_, v) => cast_transfer(*v),
        e => panic!("not a transfer: {:?}", e),
    }
}

fn cast_relation(from: (Expr, AnnRef)) -> Relation {
    if let Expr::Relation(r) = from.0 {
        *r
    } else if from.0.is_uri_like() {
        Relation::from(cast_uri(from))
    } else if let Expr::Reference(_, v) = from.0 {
        cast_relation(*v)
    } else {
        panic!("not a relation: {:?}", from.0)
    }
}

fn cast_uri(from: (Expr, AnnRef)) -> Uri {
    match from.0 {
        Expr::Uri(u) => *u,
        Expr::Relation(r) => r.uri,
        Expr::Reference(_, v) => cast_uri(*v),
        e => panic!("not a uri: {:?}", e),
    }
}

fn cast_lambda(from: (Expr, AnnRef)) -> syn::Declaration<Core> {
    match from.0 {
        Expr::Lambda(d) => d,
        Expr::Reference(_, v) => cast_lambda(*v),
        e => panic!("not a lambda: {:?}", e),
    }
}

fn eval_terminal<'a>(
    ctx: &mut Context<'a>,
    terminal: syn::Terminal<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let mut next_ann = ann.as_ref().clone();
    next_ann.extend(compose_annotations(terminal.annotations())?);
    let next_ann = AnnRef::new(next_ann);
    eval_any(ctx, terminal.inner(), next_ann)
}

fn eval_transfer<'a>(
    ctx: &mut Context<'a>,
    transfer: syn::Transfer<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let desc = ann.get_string("description");
    let summary = ann.get_string("summary");
    let tags = ann.get_enum("tags").unwrap_or_default();
    let id = ann.get_string("operationId");

    let mut methods = EnumMap::default();
    for m in transfer.methods() {
        methods[m] = true;
    }

    let domain = match transfer.domain() {
        Some(term) => cast_content(eval_terminal(ctx, term, AnnRef::default())?),
        None => Content::default(),
    };

    let ranges = cast_ranges(eval_any(ctx, transfer.range(), AnnRef::default())?);

    let params = match transfer.params() {
        Some(object) => Some(cast_object(eval_object(ctx, object, AnnRef::default())?)),
        None => None,
    };

    let xfer = Transfer {
        methods,
        domain,
        ranges,
        params,
        desc,
        summary,
        tags,
        id,
    };

    let expr = Expr::Transfer(Box::new(xfer));
    Ok((expr, ann))
}

fn eval_relation<'a>(
    ctx: &mut Context<'a>,
    relation: syn::Relation<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let uri = cast_uri(eval_terminal(ctx, relation.uri(), AnnRef::default())?);

    let mut xfers = Transfers::default();
    for x in relation.transfers() {
        let xfer = cast_transfer(eval_any(ctx, x, AnnRef::default())?);
        for (m, b) in xfer.methods {
            if b {
                xfers[m] = Some(xfer.clone());
            }
        }
    }

    let rel = Relation { uri, xfers };
    let expr = Expr::Relation(Box::new(rel));
    Ok((expr, ann))
}

fn eval_program<'a>(
    ctx: &mut Context<'a>,
    program: syn::Program<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let mut rels = IndexMap::new();
    for res in program.resources() {
        let rel = cast_relation(eval_any(ctx, res.relation(), AnnRef::default())?);
        rels.insert(rel.uri.pattern(), rel);
    }

    let refs = ctx
        .refs
        .drain(..)
        .filter_map(|(ident, (expr, ann))| {
            if expr.is_schema_like() {
                Some((ident, Reference::Schema(cast_schema((expr, ann)))))
            } else {
                None
            }
        })
        .collect();

    let spec = Spec { rels, refs };

    let expr = Expr::Spec(Box::new(spec));
    Ok((expr, ann))
}

fn eval_uri_template<'a>(
    ctx: &mut Context<'a>,
    template: syn::UriTemplate<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let example = ann.get_string("example");

    let mut path = Vec::new();
    for seg in template.segments() {
        match seg {
            syn::UriSegment::Element(elem) => {
                let s = UriSegment::Literal(elem.as_str().into());
                path.push(s);
            }
            syn::UriSegment::Variable(var) => {
                let p = cast_property(eval_any(ctx, var.inner(), AnnRef::default())?);
                let s = UriSegment::Variable(Box::new(p));
                path.push(s);
            }
        }
    }

    let params = match template.params() {
        Some(p) => Some(cast_object(eval_object(ctx, p, AnnRef::default())?)),
        None => None,
    };

    let uri = Uri {
        path,
        example,
        params,
    };

    let expr = Expr::Uri(Box::new(uri));
    Ok((expr, ann))
}

fn eval_variable<'a>(
    ctx: &mut Context<'a>,
    variable: syn::Variable<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let definition = definition(ctx.mods, variable.node()).expect("variable is not defined");

    if let Some(decl) = syn::Declaration::cast(definition) {
        if decl.has_bindings() {
            let expr = Expr::Lambda(decl);
            Ok((expr, ann))
        } else {
            let mut rhs_ann = compose_annotations(decl.annotations())?;
            rhs_ann.extend(ann.as_ref().clone());
            let rhs_ann = AnnRef::new(rhs_ann);
            let (expr, next_ann) = eval_any(ctx, decl.rhs(), rhs_ann)?;

            let ident = variable.ident();
            if ident.is_reference() {
                let value = (expr, next_ann.clone());
                ctx.refs.insert(ident.clone(), value.clone());
                let expr = Expr::Reference(ident, value.into());
                Ok((expr, next_ann))
            } else {
                Ok((expr, next_ann))
            }
        }
    } else if let Some(binding) = syn::Binding::cast(definition) {
        let scope = ctx.scopes.last().expect("scope is missing");
        let term = scope
            .get(&binding.ident())
            .expect("binding is undefined")
            .clone();
        eval_terminal(ctx, term, ann)
    } else {
        panic!(
            "expected variable definition to be either a declaration or a binding: {:?}",
            definition
        )
    }
}

fn eval_content<'a>(
    ctx: &mut Context<'a>,
    content: syn::Content<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let desc = ann.get_string("description");
    let examples = ann.get_props("examples");

    let schema = match content.body() {
        Some(body) => {
            let s = cast_schema(eval_any(ctx, body, AnnRef::default())?);
            Some(Box::new(s))
        }
        None => None,
    };

    let mut status = if schema.is_none() {
        Some(atom::HttpStatus::try_from(204).unwrap())
    } else {
        None
    };
    let mut media = None;
    let mut headers = None;
    for meta in content.meta().into_iter().flatten() {
        let rhs = eval_any(ctx, meta.rhs(), AnnRef::default())?;
        match meta.tag() {
            lex::Content::Media => media = Some(cast_string(rhs)),
            lex::Content::Headers => headers = Some(cast_object(rhs)),
            lex::Content::Status => status = Some(cast_http_status(rhs)?),
        }
    }

    let cnt = Content {
        schema,
        status,
        media,
        headers,
        desc,
        examples,
    };

    let expr = Expr::Content(Box::new(cnt));
    Ok((expr, ann))
}

fn eval_object<'a>(
    ctx: &mut Context<'a>,
    object: syn::Object<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let mut props = Vec::new();
    for prop in object.properties() {
        props.push(cast_property(eval_any(ctx, prop, AnnRef::default())?));
    }
    let obj = Object { props };
    let expr = Expr::Object(Box::new(obj));
    Ok((expr, ann))
}

fn eval_operation<'a>(
    ctx: &mut Context<'a>,
    operation: syn::VariadicOp<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let op = operation.operator();
    let expr = if op == atom::Operator::Range {
        let mut ranges = Ranges::new();
        for operand in operation.operands() {
            let c = cast_content(eval_any(ctx, operand, AnnRef::default())?);
            ranges.insert((c.status, c.media.clone()), c);
        }
        Expr::Ranges(Box::new(ranges))
    } else {
        let mut schemas = Vec::new();
        for operand in operation.operands() {
            let s = cast_schema(eval_any(ctx, operand, AnnRef::default())?);
            schemas.push(s);
        }
        let var_op = VariadicOp { op, schemas };
        Expr::VariadicOp(Box::new(var_op))
    };
    Ok((expr, ann))
}

fn eval_literal<'a>(
    _ctx: &mut Context<'a>,
    literal: syn::Literal<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let expr = match literal.kind() {
        lex::Literal::HttpStatus => {
            let lex::TokenValue::HttpStatus(status) = literal.value()
                else { panic!("expected an HTTP status") };
            Expr::HttpStatus(*status)
        }
        lex::Literal::Number => {
            let lex::TokenValue::Number(number) = literal.value()
                else { panic!("expected a number") };
            Expr::Number(*number)
        }
        lex::Literal::String => {
            let string = literal.as_str().to_owned();
            Expr::String(string)
        }
    };
    Ok((expr, ann))
}

fn eval_property<'a>(
    ctx: &mut Context<'a>,
    property: syn::Property<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let desc = ann.get_string("description");
    let required = ann.get_bool("required");

    let name = property.name();
    let schema = cast_schema(eval_any(ctx, property.rhs(), AnnRef::default())?);

    let prop = Property {
        name,
        schema,
        desc,
        required,
    };

    let expr = Expr::Property(Box::new(prop));
    Ok((expr, ann))
}

fn eval_primitive<'a>(
    _ctx: &mut Context<'a>,
    primitive: syn::Primitive<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let expr = match primitive.primitive() {
        lex::Primitive::Bool => Expr::PrimBoolean(Box::new(PrimBoolean {})),
        lex::Primitive::Int => {
            let p = PrimInteger {
                minimum: ann.get_int("minimum"),
                maximum: ann.get_int("maximum"),
                multiple_of: ann.get_int("multipleOf"),
                example: ann.get_int("example"),
            };
            Expr::PrimInteger(Box::new(p))
        }
        lex::Primitive::Num => {
            let p = PrimNumber {
                minimum: ann.get_num("minimum"),
                maximum: ann.get_num("maximum"),
                multiple_of: ann.get_num("multipleOf"),
                example: ann.get_num("example"),
            };
            Expr::PrimNumber(Box::new(p))
        }
        lex::Primitive::Str => {
            let p = PrimString {
                pattern: ann.get_string("pattern"),
                enumeration: ann.get_enum("enum").unwrap_or_default(),
                example: ann.get_string("example"),
            };
            Expr::PrimString(Box::new(p))
        }
        lex::Primitive::Uri => {
            let p = Uri {
                path: Vec::new(),
                params: None,
                example: ann.get_string("example"),
            };
            Expr::Uri(Box::new(p))
        }
    };
    Ok((expr, ann))
}

fn eval_array<'a>(
    ctx: &mut Context<'a>,
    array: syn::Array<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let schema = cast_schema(eval_any(ctx, array.inner(), AnnRef::default())?);
    let array = Array { item: schema };
    let expr = Expr::Array(Box::new(array));
    Ok((expr, ann))
}

fn eval_application<'a>(
    ctx: &mut Context<'a>,
    app: syn::Application<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    let decl = cast_lambda(eval_variable(ctx, app.lambda(), AnnRef::default())?);

    let mut scope = HashMap::new();
    for (binding, argument) in decl.bindings().zip(app.arguments()) {
        scope.insert(binding.ident(), argument);
    }

    let mut app_ann = compose_annotations(decl.annotations())?;
    app_ann.extend(ann.as_ref().clone());
    let app_ann = AnnRef::new(app_ann);

    ctx.scopes.push(scope);
    let (expr, next_ann) = eval_any(ctx, decl.rhs(), app_ann)?;
    ctx.scopes.pop();

    Ok((expr, next_ann))
}

fn eval_subexpression<'a>(
    ctx: &mut Context<'a>,
    expr: syn::SubExpression<'a, Core>,
    ann: AnnRef,
) -> Result<(Expr<'a>, AnnRef)> {
    eval_any(ctx, expr.inner(), ann)
}

fn eval_any<'a>(ctx: &mut Context<'a>, node: NRef<'a>, ann: AnnRef) -> Result<(Expr<'a>, AnnRef)> {
    if let Some(program) = syn::Program::cast(node) {
        eval_program(ctx, program, ann)
    } else if let Some(relation) = syn::Relation::cast(node) {
        eval_relation(ctx, relation, ann)
    } else if let Some(template) = syn::UriTemplate::cast(node) {
        eval_uri_template(ctx, template, ann)
    } else if let Some(variable) = syn::Variable::cast(node) {
        eval_variable(ctx, variable, ann)
    } else if let Some(term) = syn::Terminal::cast(node) {
        eval_terminal(ctx, term, ann)
    } else if let Some(content) = syn::Content::cast(node) {
        eval_content(ctx, content, ann)
    } else if let Some(object) = syn::Object::cast(node) {
        eval_object(ctx, object, ann)
    } else if let Some(operation) = syn::VariadicOp::cast(node) {
        eval_operation(ctx, operation, ann)
    } else if let Some(literal) = syn::Literal::cast(node) {
        eval_literal(ctx, literal, ann)
    } else if let Some(property) = syn::Property::cast(node) {
        eval_property(ctx, property, ann)
    } else if let Some(primitive) = syn::Primitive::cast(node) {
        eval_primitive(ctx, primitive, ann)
    } else if let Some(array) = syn::Array::cast(node) {
        eval_array(ctx, array, ann)
    } else if let Some(app) = syn::Application::cast(node) {
        eval_application(ctx, app, ann)
    } else if let Some(expr) = syn::SubExpression::cast(node) {
        eval_subexpression(ctx, expr, ann)
    } else if let Some(xfer) = syn::Transfer::cast(node) {
        eval_transfer(ctx, xfer, ann)
    } else {
        panic!("unexpected node: {:#?}", node)
    }
}

pub fn eval(mods: &ModuleSet, loc: &Locator) -> Result<Spec> {
    let ctx = &mut Context::new(mods);
    let ann = AnnRef::default();
    let (expr, _) = eval_any(ctx, mods.get(loc).unwrap().root(), ann)?;
    let Expr::Spec(spec) = expr else { panic!("expected a specification") };
    Ok(*spec)
}
