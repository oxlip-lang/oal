use anyhow::anyhow;
use ariadne::{Config, Label, Report, ReportKind, Source};
use oal_compiler::module::{Loader, ModuleSet};
use oal_compiler::tree::Tree;
use oal_model::locator::Locator;
use oal_model::span::Span;
pub use openapiv3::OpenAPI;
use serde::Deserialize;
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

/// Compile with the default document, retaining the existing Rust entry point.
#[wasm_bindgen]
pub fn compile(input: &str) -> CompilationResult {
    console_error_panic_hook::set_once();
    compilation_result(process(input, None))
}

/// Compile OAL source with a typed base document, using the CLI builder's rules.
pub fn compile_with_document(input: &str, base: OpenAPI) -> CompilationResult {
    console_error_panic_hook::set_once();
    compilation_result(process(input, Some(base)))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompilationInput {
    source: String,
}

fn parse_document_request(input: &str, doc: &str) -> anyhow::Result<(String, OpenAPI)> {
    let input: CompilationInput =
        serde_json::from_str(input).map_err(|err| anyhow!("invalid compilation input: {err}"))?;
    let base = serde_json::from_str(doc).map_err(|err| anyhow!("invalid base document: {err}"))?;
    Ok((input.source, base))
}

fn json_object(value: &JsValue, context: &str) -> anyhow::Result<String> {
    if value.is_null() || !value.is_object() || js_sys::Array::is_array(value) {
        return Err(anyhow!("invalid {context}: expected a JSON object"));
    }
    js_sys::JSON::stringify(value)
        .map_err(|_| anyhow!("invalid {context}: cannot serialize as JSON"))?
        .as_string()
        .ok_or_else(|| anyhow!("invalid {context}: cannot serialize as JSON"))
}

/// JSON-object WASM interface, complementing operation security in oal#14.
/// The historic compile(oalText) export remains unchanged.
#[wasm_bindgen(js_name = compileDocument)]
pub fn compile_document(input: js_sys::Object, doc: js_sys::Object) -> CompilationResult {
    console_error_panic_hook::set_once();
    let request = (|| {
        let input = json_object(&input, "compilation input")?;
        let doc = json_object(&doc, "base document")?;
        parse_document_request(&input, &doc)
    })();
    match request {
        Ok((input, base)) => compile_with_document(&input, base),
        Err(err) => compilation_result(Err(err)),
    }
}

fn compilation_result(result: anyhow::Result<String>) -> CompilationResult {
    match result {
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
fn process(input: &str, base: Option<OpenAPI>) -> anyhow::Result<String> {
    let loader = &mut WebLoader(input);
    let main = Locator::try_from(INPUT).unwrap();
    let mods = oal_compiler::module::load(loader, &main)?;
    let spec = oal_compiler::eval::eval(&mods)?;
    let mut builder = oal_openapi::Builder::new(spec);
    if let Some(base) = base {
        builder = builder.with_base(base);
    }
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

#[cfg(test)]
mod base_document_tests;
