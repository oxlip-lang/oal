use clap::Parser as ClapParser;

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

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    let input = std::fs::read_to_string(args.input)?;

    let prg: oal_compiler::Program = oal_syntax::parse(input)?;

    let spec = oal_compiler::evaluate(prg)?;

    let api = oal_codegen::Builder::new(spec).open_api();

    let output = serde_yaml::to_string(&api).unwrap();

    std::fs::write(args.output, output)?;

    Ok(())
}
