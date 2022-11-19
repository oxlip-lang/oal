use crate::errors::{Error, Kind};
use crate::rewrite::env::Env;
use crate::rewrite::module::{External, ModuleSet};
use crate::Result;
use oal_model::grammar::NodeCursor;
use oal_syntax::rewrite::parser::{Declaration, Import, Program, Variable};

pub fn resolve(mods: &ModuleSet) -> Result<()> {
    let mut env = Env::new();

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
                        env.declare(decl.identifier().clone(), ext);
                    }
                } else if let Some(decl) = Declaration::cast(node) {
                    env.open();
                    for binding in decl.bindings() {
                        let ext = External::new(mods.main(), binding.node());
                        env.declare(binding.as_ident(), ext);
                    }
                } else if let Some(var) = Variable::cast(node) {
                    let ident = var.as_ident();
                    if let Some(ext) = env.lookup(&ident) {
                        var.node().syntax().core_mut(|mut c| c.define(ext.clone()));
                    } else {
                        return Err(Error::new(Kind::NotInScope, "resolve").with(&ident));
                    }
                }
            }
            NodeCursor::End(node) => {
                if let Some(decl) = Declaration::cast(node) {
                    env.close();
                    let ext = External::new(mods.main(), decl.rhs());
                    env.declare(decl.identifier(), ext);
                }
            }
        }
    }

    Ok(())
}
