use crate::errors::{Error, Kind};
use crate::rewrite::env::Env;
use crate::rewrite::module::{External, ModuleSet};
use crate::Result;
use oal_model::grammar::NodeCursor;
use oal_syntax::rewrite::parser::{Declaration, Import, Program, Symbol};

pub fn resolve(mods: &ModuleSet) -> Result<()> {
    let mut env = Env::new();

    for cursor in mods.main().tree().root().traverse() {
        match cursor {
            NodeCursor::Start(node) => {
                if let Some(import) = Import::cast(node) {
                    let loc = mods.base().join(import.module())?;
                    if let Some(module) = mods.get(&loc) {
                        if let Some(program) = Program::cast(module.tree().root()) {
                            for decl in program.declarations() {
                                let ext = External::new(module, decl.rhs());
                                env.declare(decl.symbol().clone(), ext);
                            }
                        } else {
                            panic!("module root must be a program")
                        }
                    } else {
                        // All modules that are to be imported must be present in the module-set.
                        panic!("unknown module: {}", loc)
                    }
                } else if let Some(_decl) = Declaration::cast(node) {
                    env.open();
                    // TODO: declare lambda bindings if any
                } else if let Some(decl) = Declaration::cast(node) {
                    let ext = External::new(mods.main(), decl.rhs());
                    env.declare(decl.symbol(), ext);
                } else if let Some(symbol) = Symbol::cast(node) {
                    let ident = symbol.as_ident();
                    if let Some(ext) = env.lookup(&ident) {
                        symbol
                            .node()
                            .syntax()
                            .core_mut(|mut c| c.define(ext.clone()));
                    } else {
                        return Err(Error::new(Kind::NotInScope, "resolve").with(&ident));
                    }
                }
            }
            NodeCursor::End(node) => {
                if let Some(_decl) = Declaration::cast(node) {
                    env.close();
                }
            }
        }
    }

    Ok(())
}
