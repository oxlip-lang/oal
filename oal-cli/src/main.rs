use anyhow::anyhow;
use clap::Parser as ClapParser;
use oal_compiler::locator::Locator;
use oal_compiler::module::ModuleSet;
use oal_compiler::tree::Tree;

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

/// Loads and parses a source file into a concrete syntax tree.
fn loader(l: &Locator) -> anyhow::Result<Tree> {
    eprintln!("Loading module {}", l);
    let path = l
        .url
        .to_file_path()
        .map_err(|_| anyhow!("not a file path: {}", l))?;
    let input = std::fs::read_to_string(path)?;
    let tree = oal_syntax::parse(input)?;
    Ok(tree)
}

/// Compiles a program.
fn compiler(mods: &ModuleSet, l: &Locator) -> anyhow::Result<()> {
    eprintln!("Compiling module {}", l);
    oal_compiler::compile::compile(mods, l)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let base = Locator::try_from(args.input.as_path())?;

    let mods = oal_compiler::module::load(&base, loader, compiler)?;

    eprintln!("Generating API definition");

    let spec = oal_compiler::eval::eval(&mods, mods.base())?;

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
