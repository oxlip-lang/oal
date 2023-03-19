#[cfg(test)]
mod tests;

use crate::config::Config;
use crate::utf16::{utf16_index, utf16_position, Text};
use crate::{DefaultFileSystem, FileSystem};
use anyhow::anyhow;
use lsp_types::{
    Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
};
use oal_compiler::module::{Loader, ModuleSet};
use oal_compiler::spec::Spec;
use oal_compiler::tree::Tree;
use oal_model::{locator::Locator, span::Span};
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io;
use std::ops::Range;
use url::Url;

fn lsp_range(text: &str, range: Range<usize>) -> anyhow::Result<lsp_types::Range> {
    let start = utf16_position(text, range.start)?;
    let end = utf16_position(text, range.end)?;
    Ok(lsp_types::Range { start, end })
}

#[derive(Debug)]
pub struct Folder {
    uri: Url,
    config: Config,
    mods: Option<ModuleSet>,
    spec: Option<Spec>,
}

impl Folder {
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
                uri,
                config,
                mods: None,
                spec: None,
            })
        }
    }

    pub fn contains(&self, loc: &Locator) -> bool {
        self.uri
            .make_relative(loc.url())
            .map(|r| !r.starts_with(".."))
            .unwrap_or(false)
            || self.mods.as_ref().and_then(|m| m.get(loc)).is_some()
    }

    pub fn eval(&mut self, ws: &Workspace) {
        if let Ok(main) = self.config.main() {
            self.mods = oal_compiler::module::load(&ws.loader(), &main).ok();
            if let Some(ref m) = self.mods {
                self.spec = ws.eval(m).ok();
            }
        }
    }
}

pub type Diagnostics = HashMap<Locator, Vec<Diagnostic>>;

#[derive(Default)]
pub struct Workspace {
    docs: RefCell<HashMap<Url, Text>>,
    syntax_errors: RefCell<HashMap<Locator, Vec<oal_syntax::errors::Error>>>,
    compiler_error: RefCell<Option<(Locator, oal_compiler::errors::Error)>>,
    eval_error: RefCell<Option<(Locator, oal_compiler::errors::Error)>>,
}

impl Workspace {
    /// Reacts to an open file event.
    pub fn open(&self, p: DidOpenTextDocumentParams) -> anyhow::Result<Locator> {
        let loc = Locator::from(p.text_document.uri);
        let text = p.text_document.text.encode_utf16().collect();
        self.docs.borrow_mut().insert(loc.url().clone(), text);
        Ok(loc)
    }

    /// Reacts to a close file event.
    pub fn close(&self, p: DidCloseTextDocumentParams) -> anyhow::Result<Locator> {
        let loc = Locator::from(p.text_document.uri);
        self.docs.borrow_mut().remove(loc.url());
        Ok(loc)
    }

    /// Reacts to a file change event.
    pub fn change(&self, p: DidChangeTextDocumentParams) -> anyhow::Result<Locator> {
        let loc = Locator::from(p.text_document.uri);
        if let Some(text) = self.docs.borrow_mut().get_mut(loc.url()) {
            for change in p.content_changes.into_iter() {
                if let Some(r) = change.range {
                    let start = utf16_index(text, r.start)?;
                    let end = utf16_index(text, r.end)?;
                    let _ = text.splice(start..end, change.text.encode_utf16()).count();
                } else {
                    *text = change.text.encode_utf16().collect();
                }
            }
        }
        Ok(loc)
    }

    /// Evaluates a program. Resets evaluation errors.
    pub fn eval(&self, mods: &ModuleSet) -> anyhow::Result<Spec> {
        self.eval_error.replace(Default::default());
        match oal_compiler::eval::eval(mods, mods.base()) {
            Err(err) => {
                let loc = match err.span() {
                    Some(s) => s.locator().clone(),
                    None => mods.base().clone(),
                };
                self.eval_error.replace(Some((loc, err)));
                Err(anyhow!("evaluation failed"))
            }
            Ok(spec) => Ok(spec),
        }
    }

    /// Returns a program loader. Resets syntax and compilation errors.
    pub fn loader(&self) -> impl Loader<anyhow::Error> + '_ {
        self.syntax_errors.replace(Default::default());
        self.compiler_error.replace(Default::default());
        WorkspaceLoader(self)
    }

    /// Creates an LSP diagnostic from the given span and error.
    fn diagnostic<E: ToString>(&self, span: &Span, err: E) -> anyhow::Result<Diagnostic> {
        let text = self.read_file(span.locator())?;
        let range = lsp_range(&text, span.range())?;
        Ok(Diagnostic {
            message: err.to_string(),
            range,
            ..Default::default()
        })
    }

    /// Returns the diagnostics from the current workspace errors.
    pub fn diagnostics(&self) -> anyhow::Result<Diagnostics> {
        let mut diags = WorkspaceDiags::new(self);

        for (loc, errs) in self.syntax_errors.borrow().iter() {
            diags.add_syntax_errors(loc, errs)?;
        }
        if let Some((loc, err)) = self.compiler_error.borrow().as_ref() {
            diags.add_compiler_error(loc, err)?;
        }
        if let Some((loc, err)) = self.eval_error.borrow().as_ref() {
            diags.add_compiler_error(loc, err)?;
        }

        Ok(diags.collect())
    }

    /// Reads a file from the workspace.
    fn read_file(&self, loc: &Locator) -> io::Result<String> {
        match self.docs.borrow_mut().entry(loc.url().clone()) {
            Entry::Occupied(e) => {
                let file = String::from_utf16(e.get())
                    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
                Ok(file)
            }
            Entry::Vacant(e) => {
                let file = DefaultFileSystem.read_file(loc)?;
                e.insert(file.encode_utf16().collect());
                Ok(file)
            }
        }
    }

    #[allow(dead_code)]
    fn write_file(&self, loc: &Locator, buf: String) -> io::Result<()> {
        self.docs
            .borrow_mut()
            .insert(loc.url().clone(), buf.encode_utf16().collect());
        DefaultFileSystem.write_file(loc, buf)
    }
}

struct WorkspaceDiags<'a> {
    ws: &'a Workspace,
    diags: Diagnostics,
}

impl<'a> WorkspaceDiags<'a> {
    fn new(ws: &'a Workspace) -> Self {
        WorkspaceDiags {
            ws,
            diags: Default::default(),
        }
    }

    fn collect(self) -> Diagnostics {
        self.diags
    }

    fn add_syntax_errors(
        &mut self,
        loc: &Locator,
        errs: &[oal_syntax::errors::Error],
    ) -> anyhow::Result<()> {
        let mut diags = Vec::new();
        for err in errs.iter() {
            let span = match err {
                oal_syntax::errors::Error::Grammar(ref err) => err.span(),
                oal_syntax::errors::Error::Lexicon(ref err) => err.span(),
                _ => Span::new(loc.clone(), 0..0),
            };
            diags.push(self.ws.diagnostic(&span, err)?);
        }
        match self.diags.entry(loc.clone()) {
            Entry::Occupied(mut e) => {
                e.get_mut().append(&mut diags);
            }
            Entry::Vacant(e) => {
                e.insert(diags);
            }
        }
        Ok(())
    }

    fn add_compiler_error(
        &mut self,
        loc: &Locator,
        err: &oal_compiler::errors::Error,
    ) -> anyhow::Result<()> {
        let span = err
            .span()
            .cloned()
            .unwrap_or_else(|| Span::new(loc.clone(), 0..0));
        let diag = self.ws.diagnostic(&span, err)?;
        match self.diags.entry(span.locator().clone()) {
            Entry::Occupied(mut e) => {
                e.get_mut().push(diag);
            }
            Entry::Vacant(e) => {
                e.insert(vec![diag]);
            }
        }
        Ok(())
    }
}

struct WorkspaceLoader<'a>(&'a Workspace);

impl<'a> Loader<anyhow::Error> for WorkspaceLoader<'a> {
    /// Loads and parses a source file into a concrete syntax tree.
    fn load(&self, loc: Locator) -> anyhow::Result<Tree> {
        let input = self.0.read_file(&loc)?;
        let (tree, errs) = oal_syntax::parse(loc.clone(), input);
        self.0.syntax_errors.borrow_mut().insert(loc, errs);
        tree.ok_or_else(|| anyhow!("parsing failed"))
    }

    /// Compiles a program.
    fn compile(&self, mods: &ModuleSet, loc: &Locator) -> anyhow::Result<()> {
        if let Err(err) = oal_compiler::compile::compile(mods, loc) {
            let loc = match err.span() {
                Some(s) => s.locator().clone(),
                None => loc.clone(),
            };
            self.0.compiler_error.replace(Some((loc, err)));
            Err(anyhow!("compilation failed"))
        } else {
            Ok(())
        }
    }
}
