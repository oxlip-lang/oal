use anyhow::anyhow;
use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};
use clap::Parser as ClapParser;
use oal_compiler::module::ModuleSet;
use oal_compiler::tree::Tree;
use oal_model::locator::Locator;
use oal_model::span::Span;

/// Compiles a program into an OpenAPI description in YAML.
#[derive(ClapParser, Debug)]
struct Args {
    /// The path to the source program
    #[clap(short = 'i', long = "input", parse(from_os_str))]
    input: std::path::PathBuf,

    /// The path to the output OpenAPI description
    #[clap(short = 'o', long = "output", parse(from_os_str))]
    output: std::path::PathBuf,

    /// The path to a base OpenAPI description
    #[clap(short = 'b', long = "base", parse(from_os_str))]
    base: Option<std::path::PathBuf>,
}

/// Reads the file at the given location.
fn read_file(loc: &Locator) -> anyhow::Result<String> {
    let path = loc
        .url()
        .to_file_path()
        .map_err(|_| anyhow!("not a file path: {loc}"))?;
    let input = std::fs::read_to_string(path)?;
    Ok(input)
}

/// Reports an error to the standart error output.
fn report<E: ToString>(title: &str, span: Span, err: E) {
    let mut colors = ColorGenerator::new();
    let color = colors.next();
    let loc = span.locator();
    let input = read_file(loc).expect("cannot read source file");
    Report::build(ReportKind::Error, loc.clone(), span.start())
        .with_message(title)
        .with_label(Label::new(span.clone()).with_message(err).with_color(color))
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
        let msg = "parsing failed";
        let span = match err {
            oal_syntax::errors::Error::Grammar(ref err) => err.span(),
            oal_syntax::errors::Error::Lexicon(ref err) => err.span(),
            _ => Span::new(loc, 0..1),
        };
        report(msg, span, err);
        Err(anyhow::Error::msg(msg))
    } else {
        Ok(tree.unwrap())
    }
}

/// Compiles a program.
fn compiler(mods: &ModuleSet, loc: &Locator) -> anyhow::Result<()> {
    eprintln!("Compiling module {loc}");
    oal_compiler::compile::compile(mods, loc).map_err(|err| {
        let msg = "compilation failed";
        let span = match err.span() {
            Some(s) => s.clone(),
            None => Span::new(loc.clone(), 0..1),
        };
        report(msg, span, err);
        anyhow::Error::msg(msg)
    })
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let base = Locator::try_from(args.input.as_path())?;

    let mods = oal_compiler::module::load(&base, loader, compiler)?;

    eprintln!("Generating API definition");

    let spec = oal_compiler::eval::eval(&mods, mods.base()).map_err(|err| {
        let msg = "evaluation failed";
        let span = match err.span() {
            Some(s) => s.clone(),
            None => Span::new(mods.base().clone(), 0..1),
        };
        report(msg, span, err);
        anyhow::Error::msg(msg)
    })?;

    let mut builder = oal_codegen::Builder::new().with_spec(spec);

    if let Some(path) = args.base {
        let file = std::fs::File::open(path)?;
        let base = serde_yaml::from_reader(file)?;
        builder = builder.with_base(base);
    }

    let api = builder.into_openapi();

    let output = serde_yaml::to_string(&api)?;

    eprintln!("Writing OpenAPI definition to {}", args.output.display());

    std::fs::write(args.output, output)?;

    Ok(())
}
