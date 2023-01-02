use crate::env::Env;
use crate::errors::{Error, Kind, Result};
use crate::module::{External, ModuleSet};
use crate::tree::NRef;
use oal_model::grammar::NodeCursor;
use oal_model::locator::Locator;
use oal_syntax::atom::Ident;
use oal_syntax::parser::{Application, Declaration, Import, Program, Variable};

fn define(env: &mut Env, ident: Ident, node: NRef) -> Result<()> {
    if let Some(ext) = env.lookup(&ident) {
        node.syntax().core_mut().define(ext.clone());
        Ok(())
    } else {
        Err(Error::new(Kind::NotInScope, "resolve").with(&ident))
    }
}

pub fn resolve(mods: &ModuleSet, loc: &Locator) -> Result<()> {
    let env = &mut Env::new();
    let current = mods.get(loc).unwrap();

    for cursor in mods.get(loc).unwrap().tree().root().traverse() {
        match cursor {
            NodeCursor::Start(node) => {
                if let Some(import) = Import::cast(node) {
                    let loc = mods.base().join(import.module())?;
                    // All modules that are to be imported must be present in the module-set.
                    let Some(module) = mods.get(&loc) else { panic!("unknown module: {}", loc) };
                    let program =
                        Program::cast(module.tree().root()).expect("module root must be a program");
                    for decl in program.declarations() {
                        let ext = External::new(module, decl.node());
                        env.declare(decl.ident().clone(), ext);
                    }
                } else if let Some(decl) = Declaration::cast(node) {
                    env.open();
                    for binding in decl.bindings() {
                        let ext = External::new(current, binding.node());
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
                    let ext = External::new(current, node);
                    env.declare(decl.ident(), ext);
                }
            }
        }
    }

    Ok(())
}
