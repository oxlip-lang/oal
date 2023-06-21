use crate::definition::{Definition, External};
use crate::env::{Entry, Env};
use crate::errors::{Error, Kind, Result};
use crate::module::ModuleSet;
use crate::stdlib;
use crate::tree::Core;
use oal_model::grammar::{AbstractSyntaxNode, NodeCursor};
use oal_model::locator::Locator;
use oal_syntax::parser::{Declaration, Import, Program, Variable};

fn define_variable(env: &mut Env, var: Variable<'_, Core>) -> Result<()> {
    let qualifier = var.qualifier().map(|q| q.ident());
    let entry = Entry::new(var.ident(), qualifier);
    if let Some(ext) = env.lookup(&entry) {
        var.node().syntax().core_mut().define(ext.clone());
        Ok(())
    } else {
        Err(Error::new(Kind::NotInScope, "variable is not defined")
            .with(&var.ident())
            .at(var.node().span()))
    }
}

fn declare_import(
    env: &mut Env,
    mods: &ModuleSet,
    loc: &Locator,
    import: Import<'_, Core>,
) -> Result<()> {
    let other = loc.join(import.module())?;
    // All modules that are to be imported must be present in the module-set.
    let Some(module) = mods.get(&other) else { panic!("unknown module: {other}") };
    let program = Program::cast(module.root()).expect("module root must be a program");
    for decl in program.declarations() {
        let ext = External::new(module, decl.node());
        let defn = Definition::External(ext);
        let entry = Entry::new(decl.ident(), import.qualifier());
        env.declare(entry, defn);
    }
    Ok(())
}

fn open_declaration(
    env: &mut Env,
    mods: &ModuleSet,
    loc: &Locator,
    decl: Declaration<'_, Core>,
) -> Result<()> {
    env.open();
    let current = mods.get(loc).unwrap();
    for binding in decl.bindings() {
        let ext = External::new(current, binding.node());
        let defn = Definition::External(ext);
        let entry = Entry::from(binding.ident());
        env.declare(entry, defn);
    }
    Ok(())
}

fn close_declaration(
    env: &mut Env,
    mods: &ModuleSet,
    loc: &Locator,
    decl: Declaration<'_, Core>,
) -> Result<()> {
    env.close();
    let current = mods.get(loc).unwrap();
    let ext = External::new(current, decl.node());
    let defn = Definition::External(ext);
    let entry = Entry::from(decl.ident());
    if let Some(_) = env.declare(entry, defn) {
        let span = decl.identifier().node().span();
        Err(Error::new(Kind::InvalidIdentifier, "identifier already exists").at(span))
    } else {
        Ok(())
    }
}

pub fn resolve(mods: &ModuleSet, loc: &Locator) -> Result<()> {
    let env = &mut Env::new();
    stdlib::import(env)?;
    let tree = mods.get(loc).unwrap();
    for cursor in tree.root().traverse() {
        match cursor {
            NodeCursor::Start(node) => {
                if let Some(import) = Import::cast(node) {
                    declare_import(env, mods, loc, import)?;
                } else if let Some(decl) = Declaration::cast(node) {
                    open_declaration(env, mods, loc, decl)?;
                } else if let Some(var) = Variable::cast(node) {
                    define_variable(env, var)?;
                }
            }
            NodeCursor::End(node) => {
                if let Some(decl) = Declaration::cast(node) {
                    close_declaration(env, mods, loc, decl)?;
                }
            }
        }
    }

    Ok(())
}
