use crate::definition::{Definition, External};
use crate::env::{Entry, Env};
use crate::errors::{Error, Kind, Result};
use crate::module::ModuleSet;
use crate::stdlib;
use crate::tree::Core;
use oal_model::grammar::{AbstractSyntaxNode, NodeCursor};
use oal_model::locator::Locator;
use oal_syntax::parser::{Declaration, Import, Program, Recursion, Variable};
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use std::collections::{hash_map, HashMap};

pub type Graph = StableDiGraph<External, ()>;

/// A builder for the graph of dependencies between variable definitions.
#[derive(Debug, Default)]
pub struct Builder {
    /// The current (i.e. opened) definition.
    current: Option<External>,
    /// The map from definitions to graph node indices.
    externals: HashMap<External, NodeIndex>,
    /// The graph of definitions.
    graph: Graph,
}

impl Builder {
    /// Inserts a new definition.
    fn insert(&mut self, ext: External) -> NodeIndex {
        match self.externals.entry(ext.clone()) {
            hash_map::Entry::Occupied(e) => *e.get(),
            hash_map::Entry::Vacant(e) => *e.insert(self.graph.add_node(ext)),
        }
    }

    /// Opens a definition, becoming the current definition.
    pub fn open(&mut self, from: External) {
        self.current = Some(from);
    }

    /// Closes a definition.
    pub fn close(&mut self) {
        self.current = None;
    }

    /// Connects the current definition to another definition.
    pub fn connect(&mut self, to: External) {
        if let Some(from) = &self.current {
            let from_idx = self.insert(from.clone());
            let to_idx = self.insert(to);
            self.graph.add_edge(from_idx, to_idx, ());
        }
    }

    /// Returns the graph of definitions.
    pub fn graph(self) -> Graph {
        self.graph
    }
}

fn define_variable(env: &mut Env, defg: &mut Builder, var: Variable<'_, Core>) -> Result<()> {
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
    let Some(module) = mods.get(&other) else {
        panic!("unknown module: {other}")
    };
    let program = Program::cast(module.root()).expect("module root must be a program");
    for decl in program.declarations() {
        let defn = Definition::External(External::new(decl.node()));
        let entry = Entry::new(decl.ident(), import.qualifier());
        env.declare(entry, defn);
    }
    Ok(())
}

fn declare_variable(env: &mut Env, decl: Declaration<'_, Core>) -> Result<()> {
    let defn = Definition::External(External::new(decl.node()));
    let entry = Entry::from(decl.ident());
    if env.declare(entry, defn).is_some() {
        let span = decl.identifier().node().span();
        Err(Error::new(Kind::InvalidIdentifier, "identifier already exists").at(span))
    } else {
        Ok(())
    }
}

fn open_declaration(env: &mut Env, defg: &mut Builder, decl: Declaration<'_, Core>) -> Result<()> {
    env.open();
    defg.open(External::new(decl.node()));
    for binding in decl.bindings() {
        let defn = Definition::External(External::new(binding.node()));
        let entry = Entry::from(binding.ident());
        env.declare(entry, defn);
    }
    Ok(())
}

fn close_declaration(env: &mut Env, defg: &mut Builder) -> Result<()> {
    defg.close();
    env.close();
    Ok(())
}

fn open_recursion(env: &mut Env, rec: Recursion<'_, Core>) -> Result<()> {
    env.open();
    let binding = rec.binding();
    let defn = Definition::External(External::new(binding.node()));
    let entry = Entry::from(binding.ident());
    env.declare(entry, defn);
    Ok(())
}

fn close_recursion(env: &mut Env) -> Result<()> {
    env.close();
    Ok(())
}

pub fn resolve(mods: &ModuleSet, loc: &Locator) -> Result<Graph> {
    let mut defg = Builder::default();

    let env = &mut Env::new();
    stdlib::import(env)?;

    let tree = mods.get(loc).unwrap();
    let prog = Program::cast(tree.root()).expect("root should be a program");
    for import in prog.imports() {
        declare_import(env, mods, loc, import)?;
    }
    for decl in prog.declarations() {
        declare_variable(env, decl)?;
    }

    for cursor in tree.root().traverse() {
        match cursor {
            NodeCursor::Start(node) => {
                if let Some(decl) = Declaration::cast(node) {
                    open_declaration(env, &mut defg, decl)?;
                } else if let Some(var) = Variable::cast(node) {
                    define_variable(env, &mut defg, var)?;
                } else if let Some(rec) = Recursion::cast(node) {
                    open_recursion(env, rec)?;
                }
            }
            NodeCursor::End(node) => {
                if Declaration::cast(node).is_some() {
                    close_declaration(env, &mut defg)?;
                } else if Recursion::cast(node).is_some() {
                    close_recursion(env)?;
                }
            }
        }
    }

    Ok(defg.graph())
}
