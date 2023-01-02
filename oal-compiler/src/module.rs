use crate::errors::{Error, Kind};
use crate::tree::{NRef, Tree};
use oal_model::grammar::NodeIdx;
use oal_model::locator::Locator;
use oal_syntax::parser::Program;
use petgraph::algo::toposort;
use petgraph::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Module {
    loc: Locator,
    tree: Tree,
}

impl Module {
    pub fn new(loc: Locator, tree: Tree) -> Self {
        Module { loc, tree }
    }

    pub fn locator(&self) -> &Locator {
        &self.loc
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }
}

#[derive(Debug)]
pub struct ModuleSet {
    base: Locator,
    mods: HashMap<Locator, Module>,
}

impl ModuleSet {
    pub fn new(main: Module) -> Self {
        ModuleSet {
            base: main.locator().clone(),
            mods: HashMap::from([(main.locator().clone(), main)]),
        }
    }

    pub fn base(&self) -> &Locator {
        &self.base
    }

    pub fn main(&self) -> &Module {
        self.mods.get(&self.base).unwrap()
    }

    pub fn insert(&mut self, m: Module) {
        self.mods.insert(m.locator().clone(), m);
    }

    pub fn len(&self) -> usize {
        self.mods.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, l: &Locator) -> Option<&Module> {
        self.mods.get(l)
    }
}

#[derive(Clone)]
pub struct External {
    loc: Locator,
    index: NodeIdx,
}

impl External {
    pub fn new(module: &Module, node: NRef) -> Self {
        External {
            loc: module.locator().clone(),
            index: node.index(),
        }
    }

    pub fn node<'a>(&self, mods: &'a ModuleSet) -> NRef<'a> {
        if let Some(module) = mods.get(&self.loc) {
            NRef::from(module.tree(), self.index)
        } else {
            // All modules must be present in the module-set.
            panic!("unknown module: {}", self.loc)
        }
    }
}

impl std::fmt::Display for External {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}#{}", &self.loc, &self.index.to_string())
    }
}

impl std::fmt::Debug for External {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

pub trait Loader<E>: Fn(&Locator) -> std::result::Result<Tree, E>
where
    E: From<Error>,
{
}

impl<E, F> Loader<E> for F
where
    E: From<Error>,
    F: Fn(&Locator) -> std::result::Result<Tree, E>,
{
}

pub trait Compiler<E>: Fn(&ModuleSet, &Locator) -> std::result::Result<(), E>
where
    E: From<Error>,
{
}

impl<E, F> Compiler<E> for F
where
    E: From<Error>,
    F: Fn(&ModuleSet, &Locator) -> std::result::Result<(), E>,
{
}

pub fn load<E, L, C>(base: &Locator, loader: L, compiler: C) -> std::result::Result<ModuleSet, E>
where
    E: From<Error>,
    L: Loader<E>,
    C: Compiler<E>,
{
    let mut deps = HashMap::new();
    let mut graph = Graph::new();
    let mut queue = Vec::new();

    let tree = loader(base)?;
    let main = Module::new(base.clone(), tree);
    let mut mods = ModuleSet::new(main);

    let root = graph.add_node(base.clone());
    deps.insert(base.clone(), root);
    queue.push(root);

    while let Some(n) = queue.pop() {
        let loc = graph.node_weight(n).unwrap();
        let module = mods.get(loc).unwrap();

        let mut imports = Vec::new();
        let prog = Program::cast(module.tree().root()).expect("expected a program");
        for import in prog.imports() {
            let i = base.join(import.module()).map_err(Error::from)?;
            imports.push(i);
        }

        for import in imports {
            if let Some(m) = deps.get(&import) {
                graph.add_edge(n, *m, ());
            } else {
                let tree = loader(&import)?;
                let module = Module::new(import.clone(), tree);
                mods.insert(module);

                let m = graph.add_node(import.clone());
                graph.add_edge(n, m, ());
                deps.insert(import, m);
                queue.push(m);
            }
        }
    }

    let topo = toposort(&graph, None).map_err(|err| {
        let loc = graph.node_weight(err.node_id()).unwrap();
        Error::new(Kind::CycleDetected, "loading module").with(loc)
    })?;
    for node in topo {
        let loc = graph.node_weight(node).unwrap();
        compiler(&mods, loc)?;
    }

    Ok(mods)
}
