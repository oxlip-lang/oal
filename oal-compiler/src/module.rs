use crate::errors::{Error, Kind};
use crate::tree::Tree;
use oal_model::locator::Locator;
use oal_syntax::parser::Program;
use petgraph::algo::toposort;
use petgraph::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ModuleSet {
    base: Locator,
    mods: HashMap<Locator, Tree>,
}

impl ModuleSet {
    pub fn new(main: Tree) -> Self {
        ModuleSet {
            base: main.locator().clone(),
            mods: HashMap::from([(main.locator().clone(), main)]),
        }
    }

    pub fn base(&self) -> &Locator {
        &self.base
    }

    pub fn main(&self) -> &Tree {
        self.mods.get(&self.base).unwrap()
    }

    pub fn insert(&mut self, m: Tree) {
        self.mods.insert(m.locator().clone(), m);
    }

    pub fn len(&self) -> usize {
        self.mods.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, l: &Locator) -> Option<&Tree> {
        self.mods.get(l)
    }
}

pub trait Loader<E>: FnMut(Locator) -> std::result::Result<Tree, E>
where
    E: From<Error>,
{
}

impl<E, F> Loader<E> for F
where
    E: From<Error>,
    F: FnMut(Locator) -> std::result::Result<Tree, E>,
{
}

pub trait Compiler<E>: FnMut(&ModuleSet, &Locator) -> std::result::Result<(), E>
where
    E: From<Error>,
{
}

impl<E, F> Compiler<E> for F
where
    E: From<Error>,
    F: FnMut(&ModuleSet, &Locator) -> std::result::Result<(), E>,
{
}

pub fn load<E, L, C>(
    base: &Locator,
    mut loader: L,
    mut compiler: C,
) -> std::result::Result<ModuleSet, E>
where
    E: From<Error>,
    L: Loader<E>,
    C: Compiler<E>,
{
    let mut deps = HashMap::new();
    let mut graph = Graph::new();
    let mut queue = Vec::new();

    let main = loader(base.clone())?;
    let mut mods = ModuleSet::new(main);

    let root = graph.add_node(base.clone());
    deps.insert(base.clone(), root);
    queue.push(root);

    while let Some(n) = queue.pop() {
        let loc = graph.node_weight(n).unwrap();
        let module = mods.get(loc).unwrap();

        let mut imports = Vec::new();
        let prog = Program::cast(module.root()).expect("expected a program");
        for import in prog.imports() {
            let i = base.join(import.module()).map_err(Error::from)?;
            imports.push(i);
        }

        for import in imports {
            if let Some(m) = deps.get(&import) {
                graph.add_edge(*m, n, ());
            } else {
                let module = loader(import.clone())?;
                mods.insert(module);

                let m = graph.add_node(import.clone());
                graph.add_edge(m, n, ());
                deps.insert(import, m);
                queue.push(m);
            }
        }
    }

    let topo = toposort(&graph, None).map_err(|err| {
        let loc = graph.node_weight(err.node_id()).unwrap();
        Error::new(Kind::CycleDetected, "cycle in module dependencies").with(loc)
    })?;
    for node in topo {
        let loc = graph.node_weight(node).unwrap();
        compiler(&mods, loc)?;
    }

    Ok(mods)
}
