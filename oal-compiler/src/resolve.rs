use crate::defgraph::DefGraph;
use crate::definition::{Definition, External};
use crate::env::{Entry, Env};
use crate::errors::{Error, Kind, Result};
use crate::module::ModuleSet;
use crate::stdlib;
use crate::tree::Core;
use oal_model::grammar::{AbstractSyntaxNode, NodeCursor};
use oal_model::locator::Locator;
use oal_syntax::parser::{Declaration, Import, Program, Recursion, Variable};

fn define_variable(env: &mut Env, defg: &mut DefGraph, var: Variable<'_, Core>) -> Result<()> {
    let qualifier = var.qualifier().map(|q| q.ident());
    let entry = Entry::new(var.ident(), qualifier);
    if let Some(definition) = env.lookup(&entry) {
        var.node().syntax().core_mut().define(definition.clone());
        // Track dependencies among external definitions.
        if let Definition::External(to) = definition {
            defg.connect(to.clone());
        }
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

fn declare_variable(
    env: &mut Env,
    mods: &ModuleSet,
    loc: &Locator,
    decl: Declaration<'_, Core>,
) -> Result<()> {
    let current = mods.get(loc).unwrap();
    let ext = External::new(current, decl.node());
    let defn = Definition::External(ext);
    let entry = Entry::from(decl.ident());
    if env.declare(entry, defn).is_some() {
        let span = decl.identifier().node().span();
        Err(Error::new(Kind::InvalidIdentifier, "identifier already exists").at(span))
    } else {
        Ok(())
    }
}

fn open_declaration(
    env: &mut Env,
    mods: &ModuleSet,
    loc: &Locator,
    defg: &mut DefGraph,
    decl: Declaration<'_, Core>,
) -> Result<()> {
    env.open();
    let current = mods.get(loc).unwrap();
    defg.open(External::new(current, decl.node()));
    for binding in decl.bindings() {
        let ext = External::new(current, binding.node());
        let defn = Definition::External(ext);
        let entry = Entry::from(binding.ident());
        env.declare(entry, defn);
    }
    Ok(())
}

fn close_declaration(env: &mut Env, defg: &mut DefGraph) -> Result<()> {
    defg.close();
    env.close();
    Ok(())
}

fn open_recursion(
    env: &mut Env,
    mods: &ModuleSet,
    loc: &Locator,
    rec: Recursion<'_, Core>,
) -> Result<()> {
    env.open();
    let current = mods.get(loc).unwrap();
    let binding = rec.binding();
    let ext = External::new(current, binding.node());
    let defn = Definition::External(ext);
    let entry = Entry::from(binding.ident());
    env.declare(entry, defn);
    Ok(())
}

fn close_recursion(env: &mut Env) -> Result<()> {
    env.close();
    Ok(())
}

pub fn resolve(mods: &ModuleSet, loc: &Locator) -> Result<()> {
    let defg = &mut DefGraph::default();

    let env = &mut Env::new();
    stdlib::import(env)?;

    let tree = mods.get(loc).unwrap();
    let prog = Program::cast(tree.root()).expect("root should be a program");
    for import in prog.imports() {
        declare_import(env, mods, loc, import)?;
    }
    for decl in prog.declarations() {
        declare_variable(env, mods, loc, decl)?;
    }

    for cursor in tree.root().traverse() {
        match cursor {
            NodeCursor::Start(node) => {
                if let Some(decl) = Declaration::cast(node) {
                    open_declaration(env, mods, loc, defg, decl)?;
                } else if let Some(var) = Variable::cast(node) {
                    define_variable(env, defg, var)?;
                } else if let Some(rec) = Recursion::cast(node) {
                    open_recursion(env, mods, loc, rec)?;
                }
            }
            NodeCursor::End(node) => {
                if Declaration::cast(node).is_some() {
                    close_declaration(env, defg)?;
                } else if Recursion::cast(node).is_some() {
                    close_recursion(env)?;
                }
            }
        }
    }

    defg.identify_recursion(mods);

    Ok(())
}
