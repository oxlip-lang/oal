use clap::Parser as ClapParser;
use oal_compiler::{Locator, ModuleSet, Program};

/// Compiles a program into an OpenAPI description in YAML.
#[derive(ClapParser, Debug)]
struct Args {
    /// The path to the source program
    #[clap(short = 'i', long = "input", parse(from_os_str))]
    input: std::path::PathBuf,

    /// The path to the OpenAPI description
    #[clap(short = 'o', long = "output", parse(from_os_str))]
    output: std::path::PathBuf,
}

/// Loads and parses a source file into a program.
fn loader(l: &Locator) -> anyhow::Result<Program> {
    eprintln!("Loading module {}", l.display());
    let input = std::fs::read_to_string(l)?;
    let program = oal_syntax::parse(input)?;
    Ok(program)
}

/// Compiles a program.
fn compiler(mods: &ModuleSet, l: &Locator, p: Program) -> anyhow::Result<Program> {
    eprintln!("Compiling module {}", l.display());
    let program = oal_compiler::compile(mods, l, p)?;
    Ok(program)
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let main_mod = Locator::from(args.input);

    let mods = oal_compiler::load(&main_mod, loader, compiler)?;

    let program = mods.get(&main_mod).unwrap();

    eprintln!("Generating API specification");

    let spec = oal_compiler::spec::Spec::try_from(program)?;

    let api = oal_codegen::Builder::new(spec).open_api();

    let output = serde_yaml::to_string(&api)?;

    eprintln!("Writing OpenAPI definition as {}", args.output.display());

    std::fs::write(args.output, output)?;

    Ok(())
}
