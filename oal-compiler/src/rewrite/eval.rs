use super::module::ModuleSet;
use super::tree::{definition, Core, NRef};
use crate::annotation::Annotation;
use crate::errors::Result;
use crate::spec::{
    Content, Object, PrimNumber, PrimString, Property, Ranges, Relation, Schema, SchemaExpr, Spec,
    Transfer, Transfers, Uri, UriSegment,
};
use enum_map::EnumMap;
use indexmap::IndexMap;
use oal_syntax::atom;
use oal_syntax::rewrite::lexer as lex;
use oal_syntax::rewrite::parser as syn;

#[derive(Debug)]
enum Value {
    Spec(Box<Spec>),
    Uri(Box<Uri>),
    Relation(Box<Relation>),
    Transfer(Box<Transfer>),
    Content(Box<Content>),
    Object(Box<Object>),
    Ranges(Box<Ranges>),
    Property(Box<Property>),
    PrimNumber(Box<PrimNumber>),
    PrimString(Box<PrimString>),
    String(String),
    Number(u64),
    HttpStatus(atom::HttpStatus),
}

#[derive(Debug)]
struct Expr {
    value: Value,
    ann: Option<Annotation>,
}

impl Expr {
    fn annotate(&mut self, other: Annotation) {
        if let Some(a) = &mut self.ann {
            a.extend(other)
        } else {
            self.ann = Some(other)
        }
    }
}

impl From<Value> for Expr {
    fn from(value: Value) -> Self {
        Expr { value, ann: None }
    }
}

struct Context<'a> {
    mods: &'a ModuleSet,
    // TODO: keep track of references here
}

fn compose_annotations<'a, I>(mut iter: I) -> Result<Option<Annotation>>
where
    I: Iterator<Item = &'a str>,
{
    if let Some(text) = iter.next() {
        let mut ann = Annotation::try_from(text)?;
        for text in iter {
            let other = Annotation::try_from(text)?;
            ann.extend(other);
        }
        Ok(Some(ann))
    } else {
        Ok(None)
    }
}

fn cast_schema(from: Expr) -> Schema {
    let expr = match from.value {
        Value::Object(o) => SchemaExpr::Object(*o),
        Value::PrimNumber(p) => SchemaExpr::Num(*p),
        Value::PrimString(s) => SchemaExpr::Str(*s),
        _ => panic!("not a schema expression: {:?}", from),
    };

    let desc = from.ann.and_then(|a| a.get_string("description"));
    let title = None;
    let required = None;
    let examples = None;

    Schema {
        expr,
        desc,
        title,
        required,
        examples,
    }
}

fn cast_content(from: Expr) -> Content {
    match from.value {
        Value::Content(c) => *c,
        _ => Content::from(cast_schema(from)),
    }
}

fn cast_ranges(from: Expr) -> Ranges {
    match from.value {
        Value::Ranges(r) => *r,
        _ => {
            let c = cast_content(from);
            Ranges::from([((c.status, c.media.clone()), c)])
        }
    }
}

fn cast_string(from: Expr) -> String {
    match from.value {
        Value::String(s) => s,
        _ => panic!("not a string: {:?}", from),
    }
}

fn cast_property(from: Expr) -> Property {
    match from.value {
        Value::Property(p) => *p,
        _ => panic!("not a property: {:?}", from),
    }
}

fn cast_http_status(from: Expr) -> Result<atom::HttpStatus> {
    match from.value {
        Value::HttpStatus(s) => Ok(s),
        Value::Number(n) => {
            let s = atom::HttpStatus::try_from(n)?;
            Ok(s)
        }
        _ => panic!("not an HTTP status: {:?}", from),
    }
}

fn eval_terminal(ctx: &mut Context, terminal: syn::Terminal<Core>) -> Result<Expr> {
    // FIXME: annotations must be known to the inner expression
    let mut expr = eval_any(ctx, terminal.inner())?;
    if let Some(other) = compose_annotations(terminal.annotation().into_iter())? {
        expr.annotate(other)
    }
    Ok(expr)
}

fn eval_transfer(ctx: &mut Context, transfer: syn::Transfer<Core>) -> Result<Expr> {
    let mut methods = EnumMap::default();
    for m in transfer.methods() {
        methods[m] = true;
    }

    let domain = match transfer.domain() {
        Some(term) => cast_content(eval_terminal(ctx, term)?),
        None => Content::default(),
    };

    let ranges = cast_ranges(eval_any(ctx, transfer.range())?);

    // TODO: evaluate params
    let params = None;

    // TODO: evaluate annotations
    let desc = None;
    let summary = None;
    let tags = Vec::default();
    let id = None;

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

    Ok(Value::Transfer(Box::new(xfer)).into())
}

fn eval_relation(ctx: &mut Context, relation: syn::Relation<Core>) -> Result<Expr> {
    let Value::Uri(uri) = eval_terminal(ctx, relation.uri())?.value
        else { panic!("expected a URI") };

    let mut xfers = Transfers::default();

    for x in relation.transfers() {
        let Value::Transfer(xfer) = eval_transfer(ctx, x)?.value
            else { panic!("expected a transfer") };
        for (m, b) in xfer.methods {
            if b {
                xfers[m] = Some(xfer.as_ref().clone());
            }
        }
    }

    let rel = Relation { uri: *uri, xfers };

    Ok(Value::Relation(Box::new(rel)).into())
}

fn eval_program(ctx: &mut Context, program: syn::Program<Core>) -> Result<Expr> {
    let mut rels = IndexMap::new();

    for res in program.resources() {
        let Value::Relation(rel) = eval_relation(ctx, res.relation())?.value
            else { panic!("expected a relation") };
        rels.insert(rel.uri.pattern(), *rel);
    }

    let spec = Spec {
        rels,
        refs: Default::default(),
    };

    Ok(Value::Spec(Box::new(spec)).into())
}

fn eval_uri_template(ctx: &mut Context, template: syn::UriTemplate<Core>) -> Result<Expr> {
    let mut path = Vec::new();
    for seg in template.segments() {
        match seg {
            syn::UriSegment::Element(elem) => path.push(UriSegment::Literal(elem.as_str().into())),
            syn::UriSegment::Variable(var) => {
                eval_any(ctx, var.inner())?;
                todo!()
            }
        }
    }
    let uri = Uri {
        path,
        example: None,
        params: None,
    };

    Ok(Value::Uri(Box::new(uri)).into())
}

fn eval_variable(ctx: &mut Context, variable: syn::Variable<Core>) -> Result<Expr> {
    let definition = definition(ctx.mods, variable.node()).expect("variable is not defined");

    if let Some(decl) = syn::Declaration::cast(definition) {
        let mut expr = eval_any(ctx, decl.rhs())?;
        if let Some(other) = compose_annotations(decl.annotations())? {
            expr.annotate(other)
        }
        Ok(expr)
    } else if let Some(binding) = syn::Identifier::cast(definition) {
        panic!("unexpected unbound variable {}", binding.ident())
    } else {
        panic!("expected definition to be either a declaration or a binding")
    }
}

fn eval_content(ctx: &mut Context, content: syn::Content<Core>) -> Result<Expr> {
    let schema = match content.body() {
        Some(body) => Some(Box::new(cast_schema(eval_any(ctx, body)?))),
        None => None,
    };

    let mut status = None;
    let mut media = None;
    let mut headers = None;
    for meta in content.meta() {
        let rhs = eval_any(ctx, meta.rhs())?;
        match meta.tag() {
            lex::Content::Media => media = Some(cast_string(rhs)),
            lex::Content::Headers => headers = None,
            lex::Content::Status => status = Some(cast_http_status(rhs)?),
        }
    }

    let desc = None;
    let examples = None;

    let cnt = Content {
        schema,
        status,
        media,
        headers,
        desc,
        examples,
    };

    Ok(Value::Content(Box::new(cnt)).into())
}

fn eval_object(ctx: &mut Context, object: syn::Object<Core>) -> Result<Expr> {
    let mut props = Vec::new();
    for prop in object.properties() {
        props.push(cast_property(eval_any(ctx, prop)?));
    }
    let obj = Object { props };
    Ok(Value::Object(Box::new(obj)).into())
}

fn eval_operation(ctx: &mut Context, operation: syn::VariadicOp<Core>) -> Result<Expr> {
    match operation.operator() {
        lex::Operator::DoubleColon => {
            let mut ranges = Ranges::new();
            for operand in operation.operands() {
                let c = cast_content(eval_any(ctx, operand)?);
                ranges.insert((c.status, c.media.clone()), c);
            }
            Ok(Value::Ranges(Box::new(ranges)).into())
        }
        _ => todo!(),
    }
}

fn eval_literal(_ctx: &mut Context, literal: syn::Literal<Core>) -> Result<Expr> {
    match literal.kind() {
        lex::Literal::HttpStatus => {
            let lex::TokenValue::HttpStatus(status) = literal.value()
                else { panic!("expected an HTTP status") };
            Ok(Value::HttpStatus(*status).into())
        }
        lex::Literal::Number => {
            let lex::TokenValue::Number(number) = literal.value()
                else { panic!("expected a number") };
            Ok(Value::Number(*number).into())
        }
        lex::Literal::String => {
            let string = literal.as_str().to_owned();
            Ok(Value::String(string).into())
        }
    }
}

fn eval_property(ctx: &mut Context, property: syn::Property<Core>) -> Result<Expr> {
    let prop = Property {
        name: property.name(),
        schema: cast_schema(eval_any(ctx, property.rhs())?),
        desc: None,
        required: None,
    };
    Ok(Value::Property(Box::new(prop)).into())
}

fn eval_primitive(_ctx: &mut Context, primitive: syn::Primitive<Core>) -> Result<Expr> {
    let value = match primitive.primitive() {
        lex::Primitive::Num => {
            let p = PrimNumber {
                minimum: None,
                maximum: None,
                multiple_of: None,
                example: None,
            };
            Value::PrimNumber(Box::new(p))
        }
        lex::Primitive::Str => {
            let p = PrimString {
                pattern: None,
                enumeration: Default::default(),
                example: None,
            };
            Value::PrimString(Box::new(p))
        }
        lex::Primitive::Uri => todo!(),
        lex::Primitive::Bool => todo!(),
        lex::Primitive::Int => todo!(),
    };
    Ok(value.into())
}

fn eval_any(ctx: &mut Context, node: NRef) -> Result<Expr> {
    if let Some(program) = syn::Program::cast(node) {
        eval_program(ctx, program)
    } else if let Some(relation) = syn::Relation::cast(node) {
        eval_relation(ctx, relation)
    } else if let Some(template) = syn::UriTemplate::cast(node) {
        eval_uri_template(ctx, template)
    } else if let Some(variable) = syn::Variable::cast(node) {
        eval_variable(ctx, variable)
    } else if let Some(term) = syn::Terminal::cast(node) {
        eval_terminal(ctx, term)
    } else if let Some(content) = syn::Content::cast(node) {
        eval_content(ctx, content)
    } else if let Some(object) = syn::Object::cast(node) {
        eval_object(ctx, object)
    } else if let Some(operation) = syn::VariadicOp::cast(node) {
        eval_operation(ctx, operation)
    } else if let Some(literal) = syn::Literal::cast(node) {
        eval_literal(ctx, literal)
    } else if let Some(property) = syn::Property::cast(node) {
        eval_property(ctx, property)
    } else if let Some(primitive) = syn::Primitive::cast(node) {
        eval_primitive(ctx, primitive)
    } else if let Some(_app) = syn::Application::cast(node) {
        todo!("application not implemented")
    } else {
        panic!("unexpected node: {:#?}", node)
    }
}

pub fn eval(mods: &ModuleSet) -> Result<Spec> {
    let ctx = &mut Context { mods };
    let expr = eval_any(ctx, mods.main().tree().root())?;
    let Value::Spec(spec) = expr.value else { panic!("expected a specification") };
    Ok(*spec)
}
