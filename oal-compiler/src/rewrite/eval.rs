use super::module::ModuleSet;
use super::tree::{definition, Core, NRef};
use crate::annotation::Annotation;
use crate::errors::Result;
use crate::spec::{
    Content, Object, Ranges, Relation, Schema, SchemaExpr, Spec, Transfer, Transfers, Uri,
    UriSegment,
};
use enum_map::EnumMap;
use indexmap::IndexMap;
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
        _ => panic!("not a schema expression {:?}", from),
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

fn eval_terminal(ctx: &mut Context, terminal: syn::Terminal<Core>) -> Result<Expr> {
    eval_any(ctx, terminal.inner())
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

    let status = None;
    let media = None;
    let headers = None;

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

fn eval_object(_ctx: &mut Context, _object: syn::Object<Core>) -> Result<Expr> {
    let obj = Object {
        ..Default::default()
    };
    Ok(Value::Object(Box::new(obj)).into())
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
