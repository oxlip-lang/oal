pub mod dispatcher;
pub mod handlers;
pub mod state;
#[cfg(test)]
mod tests;

use crate::config::Config;
use crate::utf16::{utf16_position, utf8_index};
use crate::{DefaultFileSystem, FileSystem};
use anyhow::anyhow;
use lsp_types::{
    Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
};
use oal_compiler::module::{Loader, ModuleSet};
use oal_compiler::spec::Spec;
use oal_compiler::tree::Tree;
use oal_model::{locator::Locator, span::Span};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io;
use std::ops::Range;

/// Converts a Unicode character range to a UTF-16 range.
fn utf16_range(text: &str, range: Range<usize>) -> lsp_types::Range {
    let start = utf16_position(text, range.start);
    let end = utf16_position(text, range.end);
    lsp_types::Range { start, end }
}

/// A folder in the workspace.
#[derive(Debug)]
pub struct Folder {
    config: Config,
    mods: Option<ModuleSet>,
    spec: Option<Spec>,
}

impl Folder {
    /// Creates a new workspace folder.
    pub fn new(folder: lsp_types::WorkspaceFolder) -> anyhow::Result<Self> {
        const DEFAULT_CONFIG_FILE: &str = "oal.toml";
        if folder.uri.scheme() != "file" {
            Err(anyhow!("not a file"))
        } else {
            let mut uri = folder.uri;
            // The original URL can be a base so path_segments_mut should never fail.
            uri.path_segments_mut().unwrap().push(DEFAULT_CONFIG_FILE);
            let path = uri.to_file_path().map_err(|_| anyhow!("not a path"))?;
            let config = Config::new(Some(path.as_path()))?;
            Ok(Folder {
                config,
                mods: None,
                spec: None,
            })
        }
    }

    /// Returns the compiled modules for the folder, if any.
    pub fn modules(&self) -> Option<&ModuleSet> {
        self.mods.as_ref()
    }

    /// Returns the module identified by the given locator.
    pub fn module(&self, loc: &Locator) -> Option<&Tree> {
        self.mods.as_ref().and_then(|m| m.get(loc))
    }

    /// Checks whether the given locator belongs to the folder.
    pub fn contains(&self, loc: &Locator) -> bool {
        self.mods.as_ref().and_then(|m| m.get(loc)).is_some()
    }

    /// Evaluates a workspace folder.
    pub fn eval(&mut self, ws: &mut Workspace) {
        self.mods = None;
        self.spec = None;
        if let Ok(main) = self.config.main() {
            if let Ok(mods) = ws.load(&main) {
                self.spec = ws.eval(&mods).ok();
                self.mods = Some(mods);
            }
        }
    }
}

pub type Diagnostics = HashMap<Locator, Vec<Diagnostic>>;

/// A workspace.
#[derive(Default)]
pub struct Workspace {
    docs: HashMap<Locator, String>,
    errors: Option<Vec<(Span, String)>>,
}

impl Workspace {
    /// Reacts to an open file event.
    pub fn open(&mut self, p: DidOpenTextDocumentParams) -> anyhow::Result<Locator> {
        let loc = Locator::from(p.text_document.uri);
        self.docs.insert(loc.clone(), p.text_document.text);
        Ok(loc)
    }

    /// Reacts to a close file event.
    pub fn close(&mut self, p: DidCloseTextDocumentParams) -> anyhow::Result<Locator> {
        let loc = Locator::from(p.text_document.uri);
        self.docs.remove(&loc);
        Ok(loc)
    }

    /// Reacts to a file change event.
    pub fn change(&mut self, p: DidChangeTextDocumentParams) -> anyhow::Result<Locator> {
        let loc = Locator::from(p.text_document.uri);
        if let Some(text) = self.docs.get_mut(&loc) {
            for change in p.content_changes.into_iter() {
                if let Some(r) = change.range {
                    let start = utf8_index(text, r.start);
                    let end = utf8_index(text, r.end);
                    text.replace_range(start..end, &change.text);
                } else {
                    *text = change.text;
                }
            }
        }
        Ok(loc)
    }

    /// Loads, parses and compiles a program.
    pub fn load(&mut self, loc: &Locator) -> anyhow::Result<ModuleSet> {
        let loader = &mut WorkspaceLoader(self);
        let mods = oal_compiler::module::load(loader, loc).map_err(|err| {
            if let Ok(err) = err.downcast::<oal_compiler::errors::Error>() {
                self.log_compiler_error(loc, &err)
            }
            anyhow!("loading failed")
        })?;
        oal_compiler::compile::finalize(&mods, loc)?;
        Ok(mods)
    }

    /// Evaluates a program. Resets evaluation errors.
    pub fn eval(&mut self, mods: &ModuleSet) -> anyhow::Result<Spec> {
        match oal_compiler::eval::eval(mods, mods.base()) {
            Err(err) => {
                let loc = match err.span() {
                    Some(s) => s.locator().clone(),
                    None => mods.base().clone(),
                };
                self.log_compiler_error(&loc, &err);
                Err(anyhow!("evaluation failed"))
            }
            Ok(spec) => Ok(spec),
        }
    }

    /// Logs an error.
    fn log_error(&mut self, span: Span, err: String) {
        self.errors
            .get_or_insert_with(Default::default)
            .push((span, err));
    }

    /// Logs a collection of syntax errors.
    fn log_syntax_errors<'a>(&mut self, loc: &'a Locator, errs: &'a [oal_syntax::errors::Error]) {
        for err in errs.iter() {
            let span = match err {
                oal_syntax::errors::Error::Grammar(ref err) => err.span(),
                oal_syntax::errors::Error::Lexicon(ref err) => err.span(),
                _ => Span::new(loc.clone(), 0..0),
            };
            self.log_error(span, err.to_string())
        }
    }

    /// Logs a compiler error.
    fn log_compiler_error(&mut self, loc: &Locator, err: &oal_compiler::errors::Error) {
        let span = err
            .span()
            .cloned()
            .unwrap_or_else(|| Span::new(loc.clone(), 0..0));
        self.log_error(span, err.to_string())
    }

    /// Creates an LSP diagnostic from the given span and error.
    fn diagnostic<E: ToString>(&mut self, span: &Span, err: E) -> anyhow::Result<Diagnostic> {
        let text = self.read_file(span.locator())?;
        let range = utf16_range(&text, span.range());
        Ok(Diagnostic {
            message: err.to_string(),
            range,
            ..Default::default()
        })
    }

    /// Returns the diagnostics from the accumulated errors.
    /// Reset the workspace errors.
    pub fn diagnostics(&mut self) -> anyhow::Result<Diagnostics> {
        // Make sure diagnostics are reset on all previously opened documents.
        let mut diags = self
            .docs
            .keys()
            .map(|loc| (loc.clone(), Default::default()))
            .collect::<Diagnostics>();
        let errs = self.errors.take().unwrap_or_default();
        for (span, msg) in errs {
            let diag = self.diagnostic(&span, msg)?;
            let loc = span.locator().clone();
            match diags.entry(loc) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(diag);
                }
                Entry::Vacant(e) => {
                    e.insert(vec![diag]);
                }
            }
        }
        Ok(diags)
    }

    /// Reads a file from the workspace.
    fn read_file(&mut self, loc: &Locator) -> io::Result<String> {
        match self.docs.entry(loc.clone()) {
            Entry::Occupied(e) => Ok(e.get().clone()),
            Entry::Vacant(e) => {
                let file = DefaultFileSystem.read_file(loc)?;
                e.insert(file.clone());
                Ok(file)
            }
        }
    }
}

struct WorkspaceLoader<'a>(&'a mut Workspace);

impl<'a> Loader<anyhow::Error> for WorkspaceLoader<'a> {
    /// Loads a source file.
    fn load(&mut self, loc: &Locator) -> std::io::Result<String> {
        self.0.read_file(loc)
    }

    /// Loads and parses a source file into a concrete syntax tree.
    fn parse(&mut self, loc: Locator, input: String) -> anyhow::Result<Tree> {
        let (tree, errs) = oal_syntax::parse(loc.clone(), input);
        self.0.log_syntax_errors(&loc, &errs);
        tree.ok_or_else(|| anyhow!("parsing failed"))
    }

    /// Compiles a program.
    fn compile(&mut self, mods: &ModuleSet, loc: &Locator) -> anyhow::Result<()> {
        if let Err(err) = oal_compiler::compile::prepare(mods, loc) {
            let loc = match err.span() {
                Some(s) => s.locator().clone(),
                None => loc.clone(),
            };
            self.0.log_compiler_error(&loc, &err);
            Err(anyhow!("compilation failed"))
        } else {
            Ok(())
        }
    }
}
