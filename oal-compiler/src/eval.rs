use crate::annotation::Annotation;
use crate::errors::Result;
use crate::locator::Locator;
use crate::module::ModuleSet;
use crate::spec::{
    Array, Content, Object, PrimBoolean, PrimInteger, PrimNumber, PrimString, Property, Ranges,
    Reference, References, Relation, Schema, SchemaExpr, Spec, Transfer, Transfers, Uri,
    UriSegment, VariadicOp,
};
use crate::tree::{definition, Core, NRef};
use enum_map::EnumMap;
use indexmap::IndexMap;
use oal_syntax::atom;
use oal_syntax::lexer as lex;
use oal_syntax::parser as syn;
use std::collections::HashMap;
use std::rc::Rc;

type AnnRef = Rc<Annotation>;

#[derive(Clone, Debug)]
enum Expr {
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
    Reference(atom::Ident),
    Array(Box<Array>),
    String(String),
    Number(u64),
    HttpStatus(atom::HttpStatus),
}

struct Context<'a> {
    mods: &'a ModuleSet,
    refs: Option<References>,
    scopes: Vec<HashMap<atom::Ident, (Expr, AnnRef)>>,
}

impl<'a> Context<'a> {
    fn new(mods: &'a ModuleSet) -> Self {
        Context {
            mods,
            refs: None,
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
        Expr::Reference(r) => SchemaExpr::Ref(r),
        Expr::Relation(r) => SchemaExpr::Rel(r),
        e => panic!("not a schema expression: {:?}", e),
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
    match from.0 {
        Expr::Content(c) => *c,
        _ => Content::from(cast_schema(from)),
    }
}

fn cast_ranges(from: (Expr, AnnRef)) -> Ranges {
    match from.0 {
        Expr::Ranges(r) => *r,
        _ => {
            let c = cast_content(from);
            Ranges::from([((c.status, c.media.clone()), c)])
        }
    }
}

fn cast_string(from: (Expr, AnnRef)) -> String {
    match from.0 {
        Expr::String(s) => s,
        e => panic!("not a string: {:?}", e),
    }
}

fn cast_property(from: (Expr, AnnRef)) -> Property {
    match from.0 {
        Expr::Property(p) => *p,
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
        e => panic!("not an HTTP status: {:?}", e),
    }
}

fn cast_object(from: (Expr, AnnRef)) -> Object {
    match from.0 {
        Expr::Object(o) => *o,
        e => panic!("not an object: {:?}", e),
    }
}

fn cast_transfer(from: (Expr, AnnRef)) -> Transfer {
    match from.0 {
        Expr::Transfer(x) => *x,
        e => panic!("not a transfer: {:?}", e),
    }
}

fn cast_relation(from: (Expr, AnnRef)) -> Relation {
    match from.0 {
        Expr::Relation(r) => *r,
        _ => Relation::from(cast_uri(from)),
    }
}

fn cast_uri(from: (Expr, AnnRef)) -> Uri {
    match from.0 {
        Expr::Uri(u) => *u,
        Expr::Relation(r) => r.uri,
        e => panic!("not a uri: {:?}", e),
    }
}

fn eval_terminal(
    ctx: &mut Context,
    terminal: syn::Terminal<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
    let mut next_ann = ann.as_ref().clone();
    next_ann.extend(compose_annotations(terminal.annotations())?);
    let next_ann = AnnRef::new(next_ann);
    eval_any(ctx, terminal.inner(), next_ann)
}

fn eval_transfer(
    ctx: &mut Context,
    transfer: syn::Transfer<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
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

fn eval_relation(
    ctx: &mut Context,
    relation: syn::Relation<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
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

fn eval_program(
    ctx: &mut Context,
    program: syn::Program<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
    let mut rels = IndexMap::new();
    for res in program.resources() {
        let rel = cast_relation(eval_any(ctx, res.relation(), AnnRef::default())?);
        rels.insert(rel.uri.pattern(), rel);
    }

    let spec = Spec {
        rels,
        refs: ctx.refs.take().unwrap_or_default(),
    };

    let expr = Expr::Spec(Box::new(spec));
    Ok((expr, ann))
}

fn eval_uri_template(
    ctx: &mut Context,
    template: syn::UriTemplate<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
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

fn eval_variable(
    ctx: &mut Context,
    variable: syn::Variable<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
    let definition = definition(ctx.mods, variable.node()).expect("variable is not defined");

    if let Some(decl) = syn::Declaration::cast(definition) {
        let mut rhs_ann = compose_annotations(decl.annotations())?;
        rhs_ann.extend(ann.as_ref().clone());
        let rhs_ann = AnnRef::new(rhs_ann);
        let (expr, next_ann) = eval_any(ctx, decl.rhs(), rhs_ann)?;

        let ident = variable.ident();
        if ident.is_reference() {
            let reference = Reference::Schema(cast_schema((expr, next_ann.clone())));
            ctx.refs
                .get_or_insert_with(Default::default)
                .insert(ident.clone(), reference);
            let expr = Expr::Reference(ident);
            Ok((expr, next_ann))
        } else {
            Ok((expr, next_ann))
        }
    } else if let Some(binding) = syn::Binding::cast(definition) {
        let scope = ctx.scopes.last().expect("scope is missing");
        let (expr, bind_ann) = scope
            .get(&binding.ident())
            .expect("binding is undefined")
            .clone();
        let mut next_ann = ann.as_ref().clone();
        next_ann.extend(bind_ann.as_ref().clone());
        let next_ann = AnnRef::new(next_ann);
        Ok((expr, next_ann))
    } else {
        panic!(
            "expected variable definition to be either a declaration or a binding: {:?}",
            definition
        )
    }
}

fn eval_content(
    ctx: &mut Context,
    content: syn::Content<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
    let desc = ann.get_string("description");
    let examples = ann.get_props("examples");

    let schema = match content.body() {
        Some(body) => Some(Box::new(cast_schema(eval_any(
            ctx,
            body,
            AnnRef::default(),
        )?))),
        None => None,
    };

    let mut status = if schema.is_none() {
        Some(atom::HttpStatus::try_from(204).unwrap())
    } else {
        None
    };
    let mut media = None;
    let mut headers = None;
    for meta in content.meta() {
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

fn eval_object(
    ctx: &mut Context,
    object: syn::Object<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
    let mut props = Vec::new();
    for prop in object.properties() {
        props.push(cast_property(eval_any(ctx, prop, AnnRef::default())?));
    }
    let obj = Object { props };
    let expr = Expr::Object(Box::new(obj));
    Ok((expr, ann))
}

fn eval_operation(
    ctx: &mut Context,
    operation: syn::VariadicOp<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
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
            schemas.push(cast_schema(eval_any(ctx, operand, AnnRef::default())?));
        }
        let var_op = VariadicOp { op, schemas };
        Expr::VariadicOp(Box::new(var_op))
    };
    Ok((expr, ann))
}

fn eval_literal(
    _ctx: &mut Context,
    literal: syn::Literal<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
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

fn eval_property(
    ctx: &mut Context,
    property: syn::Property<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
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

fn eval_primitive(
    _ctx: &mut Context,
    primitive: syn::Primitive<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
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

fn eval_array(ctx: &mut Context, array: syn::Array<Core>, ann: AnnRef) -> Result<(Expr, AnnRef)> {
    let array = Array {
        item: cast_schema(eval_any(ctx, array.inner(), AnnRef::default())?),
    };
    let expr = Expr::Array(Box::new(array));
    Ok((expr, ann))
}

fn eval_application(
    ctx: &mut Context,
    app: syn::Application<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
    let definition = definition(ctx.mods, app.node()).expect("function is not defined");

    if let Some(decl) = syn::Declaration::cast(definition) {
        let mut scope = HashMap::new();
        for (binding, argument) in decl.bindings().zip(app.arguments()) {
            let arg = eval_terminal(ctx, argument, AnnRef::default())?;
            scope.insert(binding.ident(), arg);
        }

        let mut app_ann = compose_annotations(decl.annotations())?;
        app_ann.extend(ann.as_ref().clone());
        let app_ann = AnnRef::new(app_ann);

        ctx.scopes.push(scope);
        let (expr, next_ann) = eval_any(ctx, decl.rhs(), app_ann)?;
        ctx.scopes.pop();

        Ok((expr, next_ann))
    } else {
        panic!("expected function definition to be a declaration")
    }
}

fn eval_subexpression(
    ctx: &mut Context,
    expr: syn::SubExpression<Core>,
    ann: AnnRef,
) -> Result<(Expr, AnnRef)> {
    eval_any(ctx, expr.inner(), ann)
}

fn eval_any(ctx: &mut Context, node: NRef, ann: AnnRef) -> Result<(Expr, AnnRef)> {
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
    let (expr, _) = eval_any(ctx, mods.get(loc).unwrap().tree().root(), ann)?;
    let Expr::Spec(spec) = expr else { panic!("expected a specification") };
    Ok(*spec)
}
