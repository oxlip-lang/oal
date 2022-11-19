use super::env::Env;
use super::module::{External, ModuleSet};
use super::tree::NRef;
use crate::errors::{Error, Kind};
use crate::Result;
use oal_model::grammar::NodeCursor;
use oal_syntax::atom::Ident;
use oal_syntax::rewrite::parser::{Application, Declaration, Import, Program, Variable};

fn define(env: &mut Env, ident: Ident, node: NRef) -> Result<()> {
    if let Some(ext) = env.lookup(&ident) {
        node.syntax().core_mut(|mut c| c.define(ext.clone()));
        Ok(())
    } else {
        Err(Error::new(Kind::NotInScope, "resolve").with(&ident))
    }
}

pub fn resolve(mods: &ModuleSet) -> Result<()> {
    let env = &mut Env::new();

    for cursor in mods.main().tree().root().traverse() {
        match cursor {
            NodeCursor::Start(node) => {
                if let Some(import) = Import::cast(node) {
                    let loc = mods.base().join(import.module())?;
                    // All modules that are to be imported must be present in the module-set.
                    let Some(module) = mods.get(&loc) else { panic!("unknown module: {}", loc) };
                    let program =
                        Program::cast(module.tree().root()).expect("module root must be a program");
                    for decl in program.declarations() {
                        let ext = External::new(module, decl.rhs());
                        env.declare(decl.ident().clone(), ext);
                    }
                } else if let Some(decl) = Declaration::cast(node) {
                    env.open();
                    for binding in decl.bindings() {
                        let ext = External::new(mods.main(), binding.node());
                        env.declare(binding.ident(), ext);
                    }
                } else if let Some(var) = Variable::cast(node) {
                    define(env, var.ident(), var.node())?;
                } else if let Some(app) = Application::cast(node) {
                    define(env, app.ident(), app.node())?;
                }
            }
            NodeCursor::End(node) => {
                if let Some(decl) = Declaration::cast(node) {
                    env.close();
                    let ext = External::new(mods.main(), decl.rhs());
                    env.declare(decl.ident(), ext);
                }
            }
        }
    }

    Ok(())
}
