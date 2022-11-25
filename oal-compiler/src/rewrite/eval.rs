use super::module::ModuleSet;
use super::tree::{Core, NRef};
use crate::errors::Result;
use crate::spec::{Relation, Spec, Uri, UriSegment};
use indexmap::IndexMap;
use oal_syntax::rewrite::parser as syn;

// TODO: complete evaluation strategy.

enum Expr {
    Spec(Box<Spec>),
    Uri(Box<Uri>),
    Relation(Box<Relation>),
}

struct Context<'a> {
    mods: &'a ModuleSet,
}

fn eval_terminal(ctx: &mut Context, terminal: syn::Terminal<Core>) -> Expr {
    recurse(ctx, terminal.inner())
}

fn eval_relation(ctx: &mut Context, relation: syn::Relation<Core>) -> Expr {
    let Expr::Uri(uri) = eval_terminal(ctx, relation.uri())
        else { panic!("expected a URI") };

    let rel = Relation {
        uri: *uri,
        xfers: Default::default(),
    };

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
                recurse(ctx, var.inner());
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
    let definition = variable
        .node()
        .syntax()
        .core_ref()
        .definition()
        .expect("variable is not defined")
        .node(ctx.mods);
    if let Some(decl) = syn::Declaration::cast(definition) {
        recurse(ctx, decl.rhs())
    } else if let Some(binding) = syn::Identifier::cast(definition) {
        panic!("cannot evaluate the unbound variable {}", binding.ident())
    } else {
        panic!("expected definition to be either a declaration or a binding")
    }
}

fn recurse(ctx: &mut Context, node: NRef) -> Expr {
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
    } else if let Some(_app) = syn::Application::cast(node) {
        todo!("application not implemented")
    } else {
        panic!("unexpected node: {:#?}", node)
    }
}

pub fn eval(mods: &ModuleSet) -> Result<Spec> {
    let ctx = &mut Context { mods };
    let Expr::Spec(spec) = recurse(ctx, mods.main().tree().root())
        else { panic!("evaluation must return a specification") };
    Ok(*spec)
}
