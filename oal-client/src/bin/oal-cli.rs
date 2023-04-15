use oal_client::cli::Processor;
use oal_client::{config, DefaultFileSystem, FileSystem};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let proc = Processor::new();

    let config = config::Config::new(None)?;
    let main = config.main()?;
    let target = config.target()?;
    let base = config.base()?;

    let mods = oal_compiler::module::load(&mut proc.loader(), &main)?;

    eprintln!("Generating API definition");
    let spec = proc.eval(&mods)?;
    let mut builder = oal_openapi::Builder::new().with_spec(spec);

    if let Some(ref loc) = base {
        let path: PathBuf = loc.try_into()?;
        let file = std::fs::File::open(path)?;
        let base = serde_yaml::from_reader(file)?;
        builder = builder.with_base(base);
    }

    let api = builder.into_openapi();
    let api_yaml = serde_yaml::to_string(&api)?;

    eprintln!("Writing OpenAPI definition to {target}");
    DefaultFileSystem.write_file(&target, api_yaml)?;

    Ok(())
}
