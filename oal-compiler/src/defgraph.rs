use crate::definition::External;
use crate::module::ModuleSet;
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use std::collections::{hash_map, HashMap};

/// A graph of dependencies between variable definitions.
#[derive(Debug, Default)]
pub struct DefGraph {
    /// The current (i.e. opened) definition.
    current: Option<External>,
    /// The map from definitions to graph node indices.
    externals: HashMap<External, NodeIndex>,
    /// The graph of definitions.
    graph: Graph<External, ()>,
}

impl DefGraph {
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

    /// Identifies points of recursion in the graph of definitions.
    pub fn identify_recursion(&self, mods: &ModuleSet) {
        let sccs = petgraph::algo::kosaraju_scc(&self.graph);
        for component in sccs {
            // A trivial component contains a single vertex which is not connected to itself.
            // All non-trivial components contain self or mutually recursive definitions.
            let is_trivial = component.len() == 1 && {
                let idx = *component.first().unwrap();
                self.graph.find_edge(idx, idx).is_none()
            };
            for index in component {
                let ext = self.graph.node_weight(index).expect("should exist");
                let node = ext.node(mods);
                node.syntax().core_mut().is_recursive = !is_trivial;
            }
        }
    }
}
