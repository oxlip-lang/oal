use super::module::ModuleSet;
use super::tree::{definition, Core, NRef};
use crate::errors::Result;
use crate::spec::{
    Content, Object, Ranges, Relation, Schema, SchemaExpr, Spec, Transfer, Transfers, Uri,
    UriSegment,
};
use enum_map::EnumMap;
use indexmap::IndexMap;
use oal_syntax::rewrite::parser as syn;

#[derive(Debug)]
enum Expr {
    Spec(Box<Spec>),
    Uri(Box<Uri>),
    Relation(Box<Relation>),
    Transfer(Box<Transfer>),
    Content(Box<Content>),
    Object(Box<Object>),
}

struct Context<'a> {
    mods: &'a ModuleSet,
    // TODO: keep the current annotation here
    // TODO: keep track of references here
}

fn eval_terminal(ctx: &mut Context, terminal: syn::Terminal<Core>) -> Expr {
    eval_any(ctx, terminal.inner())
}

fn eval_transfer(ctx: &mut Context, transfer: syn::Transfer<Core>) -> Expr {
    let mut methods = EnumMap::default();
    for m in transfer.methods() {
        methods[m] = true;
    }

    let domain = match transfer.domain() {
        Some(term) => match eval_terminal(ctx, term) {
            Expr::Content(c) => *c,
            _ => panic!("expected a content"),
        },
        None => Content::default(),
    };

    // TODO: evaluate ranges
    let ranges = Ranges::default();

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

    Expr::Transfer(Box::new(xfer))
}

fn eval_relation(ctx: &mut Context, relation: syn::Relation<Core>) -> Expr {
    let Expr::Uri(uri) = eval_terminal(ctx, relation.uri())
        else { panic!("expected a URI") };

    let mut xfers = Transfers::default();

    for x in relation.transfers() {
        let Expr::Transfer(xfer) = eval_transfer(ctx, x) else { panic!("expected a transfer") };
        for (m, b) in xfer.methods {
            if b {
                xfers[m] = Some(xfer.as_ref().clone());
            }
        }
    }

    let rel = Relation { uri: *uri, xfers };

    Expr::Relation(Box::new(rel))
}

fn eval_program(ctx: &mut Context, program: syn::Program<Core>) -> Expr {
    let mut rels = IndexMap::new();

    for res in program.resources() {
        let Expr::Relation(rel) = eval_relation(ctx, res.relation())
            else { panic!("expected a relation") };
        rels.insert(rel.uri.pattern(), *rel);
    }

    let spec = Spec {
        rels,
        refs: Default::default(),
    };

    Expr::Spec(Box::new(spec))
}

fn eval_uri_template(ctx: &mut Context, template: syn::UriTemplate<Core>) -> Expr {
    let mut path = Vec::new();
    for seg in template.segments() {
        match seg {
            syn::UriSegment::Element(elem) => path.push(UriSegment::Literal(elem.as_str().into())),
            syn::UriSegment::Variable(var) => {
                eval_any(ctx, var.inner());
            }
        }
    }
    let uri = Uri {
        path,
        example: None,
        params: None,
    };
    Expr::Uri(Box::new(uri))
}

fn eval_variable(ctx: &mut Context, variable: syn::Variable<Core>) -> Expr {
    let definition = definition(ctx.mods, variable.node()).expect("variable is not defined");
    if let Some(decl) = syn::Declaration::cast(definition) {
        eval_any(ctx, decl.rhs())
    } else if let Some(binding) = syn::Identifier::cast(definition) {
        panic!("cannot evaluate the unbound variable {}", binding.ident())
    } else {
        panic!("expected definition to be either a declaration or a binding")
    }
}

fn cast_schema(_ctx: &mut Context, from: Expr) -> Schema {
    let expr = match from {
        Expr::Object(o) => SchemaExpr::Object(*o),
        _ => panic!("not a schema expression {:?}", from),
    };

    let desc = None;
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

fn eval_content(ctx: &mut Context, content: syn::Content<Core>) -> Expr {
    let schema = content.body().map(|body| {
        let expr = eval_any(ctx, body);
        let schema = cast_schema(ctx, expr);
        Box::new(schema)
    });

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

    Expr::Content(Box::new(cnt))
}

fn eval_object(_ctx: &mut Context, _object: syn::Object<Core>) -> Expr {
    let obj = Object {
        ..Default::default()
    };
    Expr::Object(Box::new(obj))
}

fn eval_any(ctx: &mut Context, node: NRef) -> Expr {
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
    let Expr::Spec(spec) = eval_any(ctx, mods.main().tree().root())
        else { panic!("evaluation must return a specification") };
    Ok(*spec)
}
