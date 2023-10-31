use crate::{DefaultFileSystem, FileSystem};
use anyhow::anyhow;
use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use oal_compiler::module::{Loader, ModuleSet};
use oal_compiler::spec::Spec;
use oal_compiler::tree::Tree;
use oal_model::locator::Locator;
use oal_model::span::Span;
use std::fmt::{Display, Formatter};

#[derive(Default)]
/// The CLI compilation processor.
pub struct Processor;

impl Processor {
    pub fn new() -> Self {
        Processor
    }
}

impl Processor {
    /// Reports an error.
    pub fn report<M: ToString>(&self, span: Span, msg: M) -> anyhow::Result<()> {
        let mut colors = ColorGenerator::new();
        let color = colors.next();
        let loc = span.locator().clone();
        let input = DefaultFileSystem.read_file(&loc)?;
        let mut builder =
            Report::build(ReportKind::Error, loc.clone(), span.start()).with_message(msg);
        if !span.range().is_empty() {
            let s = CharSpan::from(&input, span);
            builder.add_label(Label::new(s).with_color(color))
        }
        builder.finish().eprint((loc, Source::from(input)))?;
        Ok(())
    }

    pub fn load(&self, main: &Locator) -> anyhow::Result<ModuleSet> {
        let mods = oal_compiler::module::load(&mut self.loader(), main)?;
        Ok(mods)
    }

    /// Evaluates a program.
    pub fn eval(&self, mods: &ModuleSet) -> anyhow::Result<Spec> {
        match oal_compiler::eval::eval(mods) {
            Err(err) => {
                let span = match err.span() {
                    Some(s) => s.clone(),
                    None => Span::new(mods.base().clone(), 0..0),
                };
                self.report(span, &err)?;
                Err(anyhow!("evaluation failed"))
            }
            Ok(spec) => Ok(spec),
        }
    }

    pub fn loader(&self) -> impl Loader<anyhow::Error> + '_ {
        ProcLoader(self)
    }
}

struct ProcLoader<'a>(&'a Processor);

impl<'a> Loader<anyhow::Error> for ProcLoader<'a> {
    /// Returns true if the given locator points to a valid source file.
    fn is_valid(&mut self, loc: &Locator) -> bool {
        DefaultFileSystem.is_valid(loc)
    }

    /// Loads a source file.
    fn load(&mut self, loc: &Locator) -> anyhow::Result<String> {
        let code = DefaultFileSystem.read_file(loc)?;
        Ok(code)
    }

    /// Parses a source file into a concrete syntax tree.
    fn parse(&mut self, loc: Locator, input: String) -> anyhow::Result<Tree> {
        eprintln!("Parsing module {loc}");
        let (tree, mut errs) = oal_syntax::parse(loc.clone(), input);
        if let Some(err) = errs.pop() {
            // We don't care about error recovery for the command line interface.
            let span = match err {
                oal_syntax::errors::Error::Grammar(ref err) => err.span(),
                oal_syntax::errors::Error::Lexicon(ref err) => err.span(),
                _ => Span::new(loc, 0..0),
            };
            self.0.report(span, &err)?;
            Err(anyhow!("parsing failed"))
        } else {
            tree.ok_or_else(|| anyhow!("parsing failed"))
        }
    }

    /// Compiles a program.
    fn compile(&mut self, mods: &ModuleSet, loc: &Locator) -> anyhow::Result<()> {
        eprintln!("Compiling module {loc}");
        if let Err(err) = oal_compiler::compile::compile(mods, loc) {
            let span = match err.span() {
                Some(s) => s.clone(),
                None => Span::new(loc.clone(), 0..0),
            };
            self.0.report(span, &err)?;
            Err(anyhow!("compilation failed"))
        } else {
            Ok(())
        }
    }
}

/// A span of Unicode code points.
pub struct CharSpan {
    start: usize,
    end: usize,
    loc: Locator,
}

fn utf8_to_char_index(input: &str, index: usize) -> usize {
    let mut char_index = 0;
    for (utf8_index, _) in input.char_indices() {
        if utf8_index >= index {
            return char_index;
        }
        char_index += 1;
    }
    char_index
}

#[test]
fn test_utf8_to_char_index() {
    let input = "some😉text!";
    assert_eq!(input.len(), 13);
    assert_eq!(input.chars().count(), 10);
    assert_eq!(utf8_to_char_index(input, 0), 0);
    assert_eq!(utf8_to_char_index(input, 8), 5);
    assert_eq!(utf8_to_char_index(input, 42), 10);
}

impl CharSpan {
    fn from(input: &str, span: oal_model::span::Span) -> Self {
        CharSpan {
            start: utf8_to_char_index(input, span.start()),
            end: utf8_to_char_index(input, span.end()),
            loc: span.locator().clone(),
        }
    }
}

impl ariadne::Span for CharSpan {
    type SourceId = Locator;

    fn source(&self) -> &Self::SourceId {
        &self.loc
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

impl Display for CharSpan {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}..{}", self.loc, self.start, self.end)
    }
}
