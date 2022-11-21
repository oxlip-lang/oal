use super::module::ModuleSet;
use super::tree::{Core, NRef};
use crate::errors::Result;
use crate::spec::{Relation, Spec, Uri, UriSegment};
use indexmap::IndexMap;
use oal_syntax::rewrite::parser as syn;

// TODO: complete evaluation strategy.

pub enum Expr {
    Spec(Box<Spec>),
    Uri(Box<Uri>),
    Relation(Box<Relation>),
}

fn eval_terminal(mods: &ModuleSet, terminal: syn::Terminal<Core>) -> Expr {
    recurse(mods, terminal.inner())
}

fn eval_relation(mods: &ModuleSet, relation: syn::Relation<Core>) -> Expr {
    let Expr::Uri(uri) = eval_terminal(mods, relation.uri())
        else { panic!("expected a URI") };

    let rel = Relation {
        uri: *uri,
        xfers: Default::default(),
    };

    Expr::Relation(Box::new(rel))
}

fn eval_program(mods: &ModuleSet, program: syn::Program<Core>) -> Expr {
    let mut rels = IndexMap::new();

    for res in program.resources() {
        let Expr::Relation(rel) = eval_relation(mods, res.relation())
            else { panic!("expected a relation") };
        rels.insert(rel.uri.pattern(), *rel);
    }

    let spec = Spec {
        rels,
        refs: Default::default(),
    };

    Expr::Spec(Box::new(spec))
}

fn eval_uri_template(mods: &ModuleSet, template: syn::UriTemplate<Core>) -> Expr {
    let mut path = Vec::new();
    for seg in template.segments() {
        match seg {
            syn::UriSegment::Element(elem) => path.push(UriSegment::Literal(elem.as_str().into())),
            syn::UriSegment::Variable(var) => {
                recurse(mods, var.inner());
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

fn eval_variable(mods: &ModuleSet, variable: syn::Variable<Core>) -> Expr {
    let definition = variable
        .node()
        .syntax()
        .core_ref()
        .definition()
        .expect("variable is not defined")
        .node(mods);
    recurse(mods, definition)
}

fn recurse(mods: &ModuleSet, node: NRef) -> Expr {
    if let Some(program) = syn::Program::cast(node) {
        eval_program(mods, program)
    } else if let Some(relation) = syn::Relation::cast(node) {
        eval_relation(mods, relation)
    } else if let Some(template) = syn::UriTemplate::cast(node) {
        eval_uri_template(mods, template)
    } else if let Some(variable) = syn::Variable::cast(node) {
        eval_variable(mods, variable)
    } else if let Some(term) = syn::Terminal::cast(node) {
        eval_terminal(mods, term)
    } else if let Some(_app) = syn::Application::cast(node) {
        todo!("application not implemented")
    } else {
        panic!("unexpected node: {:#?}", node)
    }
}

pub fn eval(mods: &ModuleSet) -> Result<Spec> {
    let Expr::Spec(spec) = recurse(mods, mods.main().tree().root())
        else { panic!("evaluation must return a specification") };
    Ok(*spec)
}
