use crate::errors::{Error, Kind};
use crate::tree::Tree;
use oal_model::locator::Locator;
use oal_model::span::Span;
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

    pub fn locators(&self) -> impl Iterator<Item = &Locator> {
        self.mods.keys()
    }
}

pub trait Loader<E: From<Error>> {
    /// Loads a source file.
    fn load(&mut self, loc: &Locator) -> std::io::Result<String>;
    /// Parses a source file into a concrete syntax tree.
    fn parse(&mut self, loc: Locator, input: String) -> std::result::Result<Tree, E>;
    /// Compiles a program.
    fn compile(&mut self, mods: &ModuleSet, loc: &Locator) -> std::result::Result<(), E>;
}

pub fn load<E, L>(loader: &mut L, base: &Locator) -> std::result::Result<ModuleSet, E>
where
    E: From<Error>,
    L: Loader<E>,
{
    let mut deps = HashMap::new();
    let mut graph = Graph::new();
    let mut queue = Vec::new();

    let input = loader.load(base).map_err(Error::from)?;
    let main = loader.parse(base.clone(), input)?;
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
            let s = import.node().span();
            let i = loc
                .join(import.module())
                .map_err(|err| Error::from(err).at(s.clone()))?;
            imports.push((i, s));
        }

        for (import, span) in imports {
            if let Some(m) = deps.get(&import) {
                graph.add_edge(*m, n, ());
            } else {
                let input = loader
                    .load(&import)
                    .map_err(|err| Error::from(err).at(span))?;
                let module = loader.parse(import.clone(), input)?;
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
        Error::new(Kind::CycleDetected, "cycle in module dependencies")
            .at(Some(Span::new(loc.clone(), 0..0)))
    })?;
    for node in topo {
        let loc = graph.node_weight(node).unwrap();
        loader.compile(&mods, loc)?;
    }

    Ok(mods)
}
