use anyhow::anyhow;
use ariadne::{Config, Label, Report, ReportKind, Source};
use oal_compiler::module::{Loader, ModuleSet};
use oal_compiler::tree::Tree;
use oal_model::locator::Locator;
use oal_model::span::Span;
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;

/// The identifier for the unique source.
const INPUT: &str = "file:///main.oal";

/// The default error message if something goes very wrong.
const INTERNAL_ERRROR: &str = "internal error";

/// The result of a compilation for interfacing with JavaScript.
#[wasm_bindgen(getter_with_clone)]
pub struct CompilationResult {
    pub api: String,
    pub error: String,
}

/// The compiler interface with JavaScript.
#[wasm_bindgen]
pub fn compile(input: &str) -> CompilationResult {
    console_error_panic_hook::set_once();
    match process(input) {
        Ok(api) => CompilationResult {
            api,
            error: String::default(),
        },
        Err(err) => CompilationResult {
            api: String::default(),
            error: err.to_string(),
        },
    }
}

/// The web loader type for a unique source and no I/O.
struct WebLoader<'a>(&'a str);

impl Loader<anyhow::Error> for WebLoader<'_> {
    fn is_valid(&mut self, loc: &Locator) -> bool {
        loc.url().as_str() == INPUT
    }

    fn load(&mut self, loc: &Locator) -> anyhow::Result<String> {
        assert_eq!(loc.url().as_str(), INPUT);
        Ok(self.0.to_owned())
    }

    fn parse(&mut self, loc: Locator, input: String) -> anyhow::Result<Tree> {
        let (tree, mut errs) = oal_syntax::parse(loc.clone(), &input);
        if let Some(err) = errs.pop() {
            let span = match err {
                oal_syntax::errors::Error::Grammar(ref err) => err.span(),
                oal_syntax::errors::Error::Lexicon(ref err) => err.span(),
                _ => Span::new(loc, 0..0),
            };
            let err = report(&input, span, err).unwrap_or(INTERNAL_ERRROR.to_owned());
            Err(anyhow!(err))
        } else {
            Ok(tree.unwrap())
        }
    }

    fn compile(&mut self, mods: &ModuleSet, loc: &Locator) -> anyhow::Result<()> {
        if let Err(err) = oal_compiler::compile::compile(mods, loc) {
            let span = match err.span() {
                Some(s) => s.clone(),
                None => Span::new(loc.clone(), 0..0),
            };
            let err = report(self.0, span, err).unwrap_or(INTERNAL_ERRROR.to_owned());
            Err(anyhow!(err))
        } else {
            Ok(())
        }
    }
}

/// Runs the end-to-end compilation process on a single input.
fn process(input: &str) -> anyhow::Result<String> {
    let loader = &mut WebLoader(input);
    let main = Locator::try_from(INPUT).unwrap();
    let mods = oal_compiler::module::load(loader, &main)?;
    let spec = oal_compiler::eval::eval(&mods)?;
    let builder = oal_openapi::Builder::new(spec);
    let api = builder.into_openapi();
    let api_yaml = serde_yaml::to_string(&api)?;
    Ok(api_yaml)
}

/// Generates an error report.
fn report<M: ToString>(input: &str, span: Span, msg: M) -> anyhow::Result<String> {
    let char_span = CharSpan::from(input, span);
    let mut builder = Report::build(ReportKind::Error, char_span.clone())
        .with_config(Config::default().with_color(false))
        .with_message(msg);
    if !ariadne::Span::is_empty(&char_span) {
        builder.add_label(Label::new(char_span))
    }
    let mut buf = Vec::new();
    builder
        .finish()
        .write((INPUT, Source::from(input)), &mut buf)?;
    let out = String::from_utf8(buf)?;
    Ok(out)
}

/// A span of Unicode code points within the unique source.
#[derive(Clone, Debug)]
struct CharSpan(oal_model::span::CharSpan);

impl CharSpan {
    pub fn from(input: &str, span: Span) -> Self {
        CharSpan(oal_model::span::CharSpan::from(input, span))
    }
}

impl ariadne::Span for CharSpan {
    type SourceId = &'static str;

    fn source(&self) -> &Self::SourceId {
        &INPUT
    }

    fn start(&self) -> usize {
        self.0.start
    }

    fn end(&self) -> usize {
        self.0.end
    }
}

#[test]
fn test_compile() {
    let res = compile("res / on get -> {};");
    assert!(res.error.is_empty());
    assert!(res.api.starts_with("openapi"));
}

#[test]
fn test_compile_error() {
    let res = compile("res a on get -> {};");
    assert!(res
        .error
        .starts_with("Error: not in scope: variable is not defined"));
    assert!(res.api.is_empty());
}
