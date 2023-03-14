mod config;

use anyhow::anyhow;
use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use oal_compiler::module::ModuleSet;
use oal_compiler::tree::Tree;
use oal_model::locator::Locator;
use oal_model::span::Span;
use std::path::PathBuf;

/// Reads the file at the given location.
fn read_file(loc: &Locator) -> anyhow::Result<String> {
    let path: PathBuf = loc.try_into()?;
    let input = std::fs::read_to_string(path)?;
    Ok(input)
}

/// Writes to the file at the given location.
fn write_file(loc: &Locator, buf: String) -> anyhow::Result<()> {
    let path: PathBuf = loc.try_into()?;
    std::fs::write(path, buf)?;
    Ok(())
}

/// Reports an error to the standart error output.
fn report<E: ToString>(span: Span, err: E) {
    let mut colors = ColorGenerator::new();
    let color = colors.next();
    let loc = span.locator();
    let input = read_file(loc).expect("cannot read source file");
    Report::build(ReportKind::Error, loc.clone(), span.start())
        .with_message(err)
        .with_label(Label::new(span.clone()).with_color(color))
        .finish()
        .eprint((loc.clone(), Source::from(input)))
        .unwrap();
}

/// Loads and parses a source file into a concrete syntax tree.
fn loader(loc: Locator) -> anyhow::Result<Tree> {
    eprintln!("Loading module {loc}");
    let input = read_file(&loc)?;
    let (tree, mut errs) = oal_syntax::parse(loc.clone(), input);
    if let Some(err) = errs.pop() {
        // We don't care about error recovery for the command line interface.
        let span = match err {
            oal_syntax::errors::Error::Grammar(ref err) => err.span(),
            oal_syntax::errors::Error::Lexicon(ref err) => err.span(),
            _ => Span::new(loc, 0..1),
        };
        report(span, &err);
        Err(anyhow!("parsing failed"))
    } else {
        Ok(tree.unwrap())
    }
}

/// Compiles a program.
fn compiler(mods: &ModuleSet, loc: &Locator) -> anyhow::Result<()> {
    eprintln!("Compiling module {loc}");
    oal_compiler::compile::compile(mods, loc).map_err(|err| {
        let span = match err.span() {
            Some(s) => s.clone(),
            None => Span::new(loc.clone(), 0..1),
        };
        report(span, &err);
        anyhow!("compilation failed")
    })
}

fn main() -> anyhow::Result<()> {
    let config = config::Config::new()?;
    let main = config.main()?;
    let target = config.target()?;
    let base = config.base()?;

    let mods = oal_compiler::module::load(&main, loader, compiler)?;

    eprintln!("Generating API definition");
    let spec = oal_compiler::eval::eval(&mods, mods.base()).map_err(|err| {
        let span = match err.span() {
            Some(s) => s.clone(),
            None => Span::new(mods.base().clone(), 0..1),
        };
        report(span, &err);
        anyhow!("evaluation failed")
    })?;

    let mut builder = oal_codegen::Builder::new().with_spec(spec);

    if let Some(ref loc) = base {
        let path: PathBuf = loc.try_into()?;
        let file = std::fs::File::open(path)?;
        let base = serde_yaml::from_reader(file)?;
        builder = builder.with_base(base);
    }

    let api = builder.into_openapi();
    let api_yaml = serde_yaml::to_string(&api)?;

    eprintln!("Writing OpenAPI definition to {target}");
    write_file(&target, api_yaml)?;

    Ok(())
}
