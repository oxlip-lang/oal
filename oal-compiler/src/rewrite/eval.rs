use super::{module::ModuleSet, tree::NRef};
use crate::errors::Result;
use oal_syntax::rewrite::parser::{
    Application, Program, Terminal, UriSegment, UriTemplate, Variable,
};

// TODO: complete evaluation strategy.

pub enum Expr {
    Spec,
    Uri,
}

fn recurse(mods: &ModuleSet, node: NRef) -> Expr {
    if let Some(prog) = Program::cast(node) {
        for res in prog.resources() {
            let uri = res.relation().uri();
            recurse(mods, uri.inner());
        }
        Expr::Spec
    } else if let Some(uri) = UriTemplate::cast(node) {
        for seg in uri.segments() {
            match seg {
                UriSegment::Element(_elem) => {}
                UriSegment::Variable(var) => {
                    recurse(mods, var.inner());
                }
            }
        }
        Expr::Uri
    } else if let Some(var) = Variable::cast(node) {
        let node = var
            .node()
            .syntax()
            .core_ref()
            .definition()
            .expect("expected a definition")
            .node(mods);
        recurse(mods, node)
    } else if let Some(term) = Terminal::cast(node) {
        recurse(mods, term.inner())
    } else if let Some(_app) = Application::cast(node) {
        todo!("application not implemented")
    } else {
        panic!("unexpected node: {:#?}", node)
    }
}

pub fn eval(mods: &ModuleSet) -> Result<Expr> {
    Ok(recurse(mods, mods.main().tree().root()))
}
