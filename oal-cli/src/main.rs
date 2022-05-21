use anyhow::anyhow;
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
    eprintln!("Loading module {}", l);
    let path = l
        .url
        .to_file_path()
        .map_err(|_| anyhow!("not a file path: {}", l))?;
    let input = std::fs::read_to_string(path)?;
    let program = oal_syntax::parse(input)?;
    Ok(program)
}

/// Compiles a program.
fn compiler(mods: &ModuleSet, l: &Locator, p: Program) -> anyhow::Result<Program> {
    eprintln!("Compiling module {}", l);
    let program = oal_compiler::compile(mods, l, p)?;
    Ok(program)
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let main_mod = Locator::try_from(args.input.as_path())?;

    let mods = oal_compiler::load(&main_mod, loader, compiler)?;

    let program = mods.programs.get(&main_mod).unwrap();

    eprintln!("Generating API specification");

    let spec = oal_compiler::spec::Spec::try_from(program)?;

    let api = oal_codegen::Builder::new(spec).open_api();

    let output = serde_yaml::to_string(&api)?;

    eprintln!("Writing OpenAPI definition as {}", args.output.display());

    std::fs::write(args.output, output)?;

    Ok(())
}
