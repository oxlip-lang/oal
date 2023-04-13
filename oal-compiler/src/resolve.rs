use crate::definition::{Definition, External};
use crate::env::Env;
use crate::errors::{Error, Kind, Result};
use crate::module::ModuleSet;
use crate::stdlib;
use crate::tree::NRef;
use oal_model::grammar::NodeCursor;
use oal_model::locator::Locator;
use oal_syntax::atom::Ident;
use oal_syntax::parser::{Declaration, Import, Program, Variable};

fn define(env: &mut Env, ident: Ident, node: NRef) -> Result<()> {
    if let Some(ext) = env.lookup(&ident) {
        node.syntax().core_mut().define(ext.clone());
        Ok(())
    } else {
        Err(Error::new(Kind::NotInScope, "variable is not defined")
            .with(&ident)
            .at(node.span()))
    }
}

pub fn resolve(mods: &ModuleSet, loc: &Locator) -> Result<()> {
    let env = &mut Env::new();
    stdlib::import(env);
    let current = mods.get(loc).unwrap();

    for cursor in current.root().traverse() {
        match cursor {
            NodeCursor::Start(node) => {
                if let Some(import) = Import::cast(node) {
                    let other = loc.join(import.module())?;
                    // All modules that are to be imported must be present in the module-set.
                    let Some(module) = mods.get(&other) else { panic!("unknown module: {other}") };
                    let program =
                        Program::cast(module.root()).expect("module root must be a program");
                    for decl in program.declarations() {
                        let ext = External::new(module, decl.node());
                        let defn = Definition::External(ext);
                        env.declare(decl.ident().clone(), defn);
                    }
                } else if let Some(decl) = Declaration::cast(node) {
                    env.open();
                    for binding in decl.bindings() {
                        let ext = External::new(current, binding.node());
                        let defn = Definition::External(ext);
                        env.declare(binding.ident(), defn);
                    }
                } else if let Some(var) = Variable::cast(node) {
                    define(env, var.ident(), var.node())?;
                }
            }
            NodeCursor::End(node) => {
                if let Some(decl) = Declaration::cast(node) {
                    env.close();
                    let ext = External::new(current, node);
                    let defn = Definition::External(ext);
                    env.declare(decl.ident(), defn);
                }
            }
        }
    }

    Ok(())
}
