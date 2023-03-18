pub mod config;

use anyhow::anyhow;
use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use oal_compiler::module::{Loader, ModuleSet};
use oal_compiler::spec::Spec;
use oal_compiler::tree::Tree;
use oal_model::{locator::Locator, span::Span};
use std::path::PathBuf;

/// Reads the file at the given location.
pub fn read_file(loc: &Locator) -> anyhow::Result<String> {
    let path: PathBuf = loc.try_into()?;
    let input = std::fs::read_to_string(path)?;
    Ok(input)
}

/// Writes to the file at the given location.
pub fn write_file(loc: &Locator, buf: String) -> anyhow::Result<()> {
    let path: PathBuf = loc.try_into()?;
    std::fs::write(path, buf)?;
    Ok(())
}

/// Reports an error to the standard error output.
pub fn report<W: std::io::Write, E: ToString>(
    console: W,
    span: Span,
    err: E,
) -> anyhow::Result<()> {
    let mut colors = ColorGenerator::new();
    let color = colors.next();
    let loc = span.locator();
    let input = read_file(loc).expect("cannot read source file");
    let builder = Report::build(ReportKind::Error, loc.clone(), span.start()).with_message(err);
    let builder = if span.range().is_empty() {
        builder
    } else {
        builder.with_label(Label::new(span.clone()).with_color(color))
    };
    let report = builder.finish();
    report.write((loc.clone(), Source::from(input)), console)?;
    Ok(())
}

/// The client compilation context.
pub struct Context<W> {
    console: W,
}

impl<W> Context<W> {
    pub fn new(w: W) -> Self {
        Context { console: w }
    }

    pub fn console(&mut self) -> &mut W {
        &mut self.console
    }
}

impl<W: std::io::Write> Context<W> {
    /// Evaluates a program.
    pub fn eval(&mut self, mods: &ModuleSet) -> anyhow::Result<Spec> {
        match oal_compiler::eval::eval(mods, mods.base()) {
            Err(err) => {
                let span = match err.span() {
                    Some(s) => s.clone(),
                    None => Span::new(mods.base().clone(), 0..0),
                };
                report(self.console(), span, &err)?;
                Err(anyhow!("evaluation failed"))
            }
            Ok(spec) => Ok(spec),
        }
    }
}

impl<W: std::io::Write> Loader<anyhow::Error> for Context<W> {
    /// Loads and parses a source file into a concrete syntax tree.
    fn load(&mut self, loc: Locator) -> anyhow::Result<Tree> {
        writeln!(self.console(), "Loading module {loc}")?;
        let input = read_file(&loc)?;
        let (tree, mut errs) = oal_syntax::parse(loc.clone(), input);
        if let Some(err) = errs.pop() {
            // We don't care about error recovery for the command line interface.
            let span = match err {
                oal_syntax::errors::Error::Grammar(ref err) => err.span(),
                oal_syntax::errors::Error::Lexicon(ref err) => err.span(),
                _ => Span::new(loc, 0..0),
            };
            report(self.console(), span, &err)?;
            Err(anyhow!("parsing failed"))
        } else {
            Ok(tree.unwrap())
        }
    }

    /// Compiles a program.
    fn compile(&mut self, mods: &ModuleSet, loc: &Locator) -> anyhow::Result<()> {
        writeln!(self.console(), "Compiling module {loc}")?;
        if let Err(err) = oal_compiler::compile::compile(mods, loc) {
            let span = match err.span() {
                Some(s) => s.clone(),
                None => Span::new(loc.clone(), 0..0),
            };
            report(self.console(), span, &err)?;
            Err(anyhow!("compilation failed"))
        } else {
            Ok(())
        }
    }
}
