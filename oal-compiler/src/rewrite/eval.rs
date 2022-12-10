use super::module::ModuleSet;
use super::tree::{definition, Core, NRef};
use crate::annotation::Annotation;
use crate::errors::Result;
use crate::spec::{
    Array, Content, Object, PrimBoolean, PrimNumber, PrimString, Property, Ranges, Reference,
    References, Relation, Schema, SchemaExpr, Spec, Transfer, Transfers, Uri, UriSegment,
    VariadicOp,
};
use enum_map::EnumMap;
use indexmap::IndexMap;
use oal_syntax::atom;
use oal_syntax::rewrite::lexer as lex;
use oal_syntax::rewrite::parser as syn;

// TODO: we might not need to box all the composite values.
#[derive(Debug)]
enum Expr {
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
    ann: Option<Annotation>,
    refs: Option<References>,
}

impl<'a> Context<'a> {
    fn annotate(&mut self, other: Annotation) {
        if let Some(a) = &mut self.ann {
            a.extend(other)
        } else {
            self.ann = Some(other)
        }
    }

    fn annotation(&mut self) -> Option<Annotation> {
        self.ann.take()
    }
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

fn cast_schema(ctx: &mut Context, from: Expr) -> Schema {
    let ann = ctx.annotation();
    let desc = ann.as_ref().and_then(|a| a.get_string("description"));
    let title = ann.as_ref().and_then(|a| a.get_string("title"));
    let required = ann.as_ref().and_then(|a| a.get_bool("required"));
    let examples = ann.as_ref().and_then(|a| a.get_props("examples"));

    let expr = match from {
        Expr::Object(o) => SchemaExpr::Object(*o),
        Expr::PrimNumber(p) => SchemaExpr::Num(*p),
        Expr::PrimString(s) => SchemaExpr::Str(*s),
        Expr::PrimBoolean(b) => SchemaExpr::Bool(*b),
        Expr::Array(a) => SchemaExpr::Array(a),
        Expr::Uri(u) => SchemaExpr::Uri(*u),
        Expr::VariadicOp(o) => SchemaExpr::Op(*o),
        Expr::Reference(r) => SchemaExpr::Ref(r),
        _ => panic!("not a schema expression: {:?}", from),
    };

    Schema {
        expr,
        desc,
        title,
        required,
        examples,
    }
}

fn cast_content(ctx: &mut Context, from: Expr) -> Content {
    match from {
        Expr::Content(c) => *c,
        _ => Content::from(cast_schema(ctx, from)),
    }
}

fn cast_ranges(ctx: &mut Context, from: Expr) -> Ranges {
    match from {
        Expr::Ranges(r) => *r,
        _ => {
            let c = cast_content(ctx, from);
            Ranges::from([((c.status, c.media.clone()), c)])
        }
    }
}

fn cast_string(from: Expr) -> String {
    match from {
        Expr::String(s) => s,
        _ => panic!("not a string: {:?}", from),
    }
}

fn cast_property(from: Expr) -> Property {
    match from {
        Expr::Property(p) => *p,
        _ => panic!("not a property: {:?}", from),
    }
}

fn cast_http_status(from: Expr) -> Result<atom::HttpStatus> {
    match from {
        Expr::HttpStatus(s) => Ok(s),
        Expr::Number(n) => {
            let s = atom::HttpStatus::try_from(n)?;
            Ok(s)
        }
        _ => panic!("not an HTTP status: {:?}", from),
    }
}

fn cast_object(from: Expr) -> Object {
    match from {
        Expr::Object(o) => *o,
        _ => panic!("not an object: {:?}", from),
    }
}

fn eval_terminal(ctx: &mut Context, terminal: syn::Terminal<Core>) -> Result<Expr> {
    if let Some(other) = compose_annotations(terminal.annotation().into_iter())? {
        ctx.annotate(other)
    }
    eval_any(ctx, terminal.inner())
}

fn eval_transfer(ctx: &mut Context, transfer: syn::Transfer<Core>) -> Result<Expr> {
    let ann = ctx.annotation();
    let desc = ann.as_ref().and_then(|a| a.get_string("description"));
    let summary = ann.as_ref().and_then(|a| a.get_string("summary"));
    let tags = ann
        .as_ref()
        .and_then(|a| a.get_enum("tags"))
        .unwrap_or_default();
    let id = ann.as_ref().and_then(|a| a.get_string("operationId"));

    let mut methods = EnumMap::default();
    for m in transfer.methods() {
        methods[m] = true;
    }

    let domain = match transfer.domain() {
        Some(term) => {
            let c = eval_terminal(ctx, term)?;
            cast_content(ctx, c)
        }
        None => Content::default(),
    };

    let ranges = {
        let r = eval_any(ctx, transfer.range())?;
        cast_ranges(ctx, r)
    };

    let params = match transfer.params() {
        Some(object) => Some(cast_object(eval_object(ctx, object)?)),
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

    Ok(Expr::Transfer(Box::new(xfer)))
}

fn eval_relation(ctx: &mut Context, relation: syn::Relation<Core>) -> Result<Expr> {
    let Expr::Uri(uri) = eval_terminal(ctx, relation.uri())?
        else { panic!("expected a URI") };

    let mut xfers = Transfers::default();

    for x in relation.transfers() {
        let Expr::Transfer(xfer) = eval_transfer(ctx, x)?
            else { panic!("expected a transfer") };
        for (m, b) in xfer.methods {
            if b {
                xfers[m] = Some(xfer.as_ref().clone());
            }
        }
    }

    let rel = Relation { uri: *uri, xfers };

    Ok(Expr::Relation(Box::new(rel)))
}

fn eval_program(ctx: &mut Context, program: syn::Program<Core>) -> Result<Expr> {
    let mut rels = IndexMap::new();
    for res in program.resources() {
        let Expr::Relation(rel) = eval_relation(ctx, res.relation())?
            else { panic!("expected a relation") };
        rels.insert(rel.uri.pattern(), *rel);
    }

    let spec = Spec {
        rels,
        refs: ctx.refs.take().unwrap_or_default(),
    };

    Ok(Expr::Spec(Box::new(spec)))
}

fn eval_uri_template(ctx: &mut Context, template: syn::UriTemplate<Core>) -> Result<Expr> {
    let ann = ctx.annotation();
    let example = ann.as_ref().and_then(|a| a.get_string("example"));

    let mut path = Vec::new();
    for seg in template.segments() {
        match seg {
            syn::UriSegment::Element(elem) => {
                let s = UriSegment::Literal(elem.as_str().into());
                path.push(s);
            }
            syn::UriSegment::Variable(var) => {
                let p = eval_any(ctx, var.inner())?;
                let s = UriSegment::Variable(Box::new(cast_property(p)));
                path.push(s);
            }
        }
    }

    let params = match template.params() {
        Some(p) => Some(cast_object(eval_object(ctx, p)?)),
        None => None,
    };

    let uri = Uri {
        path,
        example,
        params,
    };

    Ok(Expr::Uri(Box::new(uri)))
}

fn eval_variable(ctx: &mut Context, variable: syn::Variable<Core>) -> Result<Expr> {
    let definition = definition(ctx.mods, variable.node()).expect("variable is not defined");

    if let Some(decl) = syn::Declaration::cast(definition) {
        if let Some(other) = compose_annotations(decl.annotations())? {
            ctx.annotate(other)
        }
        let expr = eval_any(ctx, decl.rhs())?;
        let ident = variable.ident();
        if ident.is_reference() {
            let reference = Reference::Schema(cast_schema(ctx, expr));
            ctx.refs
                .get_or_insert_with(Default::default)
                .insert(ident.clone(), reference);
            Ok(Expr::Reference(ident))
        } else {
            Ok(expr)
        }
    } else if let Some(binding) = syn::Identifier::cast(definition) {
        panic!("unexpected unbound variable {}", binding.ident())
    } else {
        panic!("expected definition to be either a declaration or a binding")
    }
}

fn eval_content(ctx: &mut Context, content: syn::Content<Core>) -> Result<Expr> {
    let ann = ctx.annotation();
    let desc = ann.as_ref().and_then(|a| a.get_string("description"));
    let examples = ann.as_ref().and_then(|a| a.get_props("examples"));

    let schema = match content.body() {
        Some(body) => {
            let s = eval_any(ctx, body)?;
            Some(Box::new(cast_schema(ctx, s)))
        }
        None => None,
    };

    let mut status = None;
    let mut media = None;
    let mut headers = None;
    for meta in content.meta() {
        let rhs = eval_any(ctx, meta.rhs())?;
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

    Ok(Expr::Content(Box::new(cnt)))
}

fn eval_object(ctx: &mut Context, object: syn::Object<Core>) -> Result<Expr> {
    let mut props = Vec::new();
    for prop in object.properties() {
        props.push(cast_property(eval_any(ctx, prop)?));
    }
    let obj = Object { props };
    Ok(Expr::Object(Box::new(obj)))
}

fn eval_operation(ctx: &mut Context, operation: syn::VariadicOp<Core>) -> Result<Expr> {
    match operation.operator() {
        lex::Operator::DoubleColon => {
            let mut ranges = Ranges::new();
            for operand in operation.operands() {
                let o = eval_any(ctx, operand)?;
                let c = cast_content(ctx, o);
                ranges.insert((c.status, c.media.clone()), c);
            }
            Ok(Expr::Ranges(Box::new(ranges)))
        }
        lex::Operator::Tilde => {
            let mut schemas = Vec::new();
            for operand in operation.operands() {
                let o = eval_any(ctx, operand)?;
                schemas.push(cast_schema(ctx, o));
            }
            let op = atom::Operator::Any;
            let var_op = VariadicOp { op, schemas };
            Ok(Expr::VariadicOp(Box::new(var_op)))
        }
        _ => todo!(),
    }
}

fn eval_literal(_ctx: &mut Context, literal: syn::Literal<Core>) -> Result<Expr> {
    match literal.kind() {
        lex::Literal::HttpStatus => {
            let lex::TokenValue::HttpStatus(status) = literal.value()
                else { panic!("expected an HTTP status") };
            Ok(Expr::HttpStatus(*status))
        }
        lex::Literal::Number => {
            let lex::TokenValue::Number(number) = literal.value()
                else { panic!("expected a number") };
            Ok(Expr::Number(*number))
        }
        lex::Literal::String => {
            let string = literal.as_str().to_owned();
            Ok(Expr::String(string))
        }
    }
}

fn eval_property(ctx: &mut Context, property: syn::Property<Core>) -> Result<Expr> {
    let ann = ctx.annotation();
    let desc = ann.as_ref().and_then(|a| a.get_string("description"));
    let required = ann.as_ref().and_then(|a| a.get_bool("required"));

    let name = property.name();
    let s = eval_any(ctx, property.rhs())?;
    let schema = cast_schema(ctx, s);

    let prop = Property {
        name,
        schema,
        desc,
        required,
    };

    Ok(Expr::Property(Box::new(prop)))
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
            Expr::PrimNumber(Box::new(p))
        }
        lex::Primitive::Str => {
            let p = PrimString {
                pattern: None,
                enumeration: Default::default(),
                example: None,
            };
            Expr::PrimString(Box::new(p))
        }
        lex::Primitive::Uri => {
            let p = Uri {
                path: Default::default(),
                params: None,
                example: None,
            };
            Expr::Uri(Box::new(p))
        }
        lex::Primitive::Bool => Expr::PrimBoolean(Box::new(PrimBoolean {})),
        lex::Primitive::Int => todo!(),
    };
    Ok(value)
}

fn eval_array(ctx: &mut Context, array: syn::Array<Core>) -> Result<Expr> {
    let item = eval_any(ctx, array.inner())?;
    let array = Array {
        item: cast_schema(ctx, item),
    };
    Ok(Expr::Array(Box::new(array)))
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
    } else if let Some(array) = syn::Array::cast(node) {
        eval_array(ctx, array)
    } else if let Some(_app) = syn::Application::cast(node) {
        todo!("application not implemented")
    } else {
        panic!("unexpected node: {:#?}", node)
    }
}

pub fn eval(mods: &ModuleSet) -> Result<Spec> {
    let ctx = &mut Context {
        mods,
        ann: None,
        refs: None,
    };
    let expr = eval_any(ctx, mods.main().tree().root())?;
    let Expr::Spec(spec) = expr else { panic!("expected a specification") };
    Ok(*spec)
}
