use log::{debug, error, info};
use oal_client::cli::Processor;
use oal_client::{config, DefaultFileSystem, FileSystem};
use std::process::ExitCode;

fn run(config: config::Config) -> anyhow::Result<()> {
    let main = config.main()?;
    let target = config.target()?;
    let base = config.base()?;

    let proc = Processor::new();
    let mods = proc.load(&main)?;

    debug!("Generating API definition");
    let spec = proc.eval(&mods)?;
    let mut builder = oal_openapi::Builder::new(spec);

    if let Some(ref loc) = base {
        let file = DefaultFileSystem.open_file(loc)?;
        let base = serde_yaml::from_reader(file)?;
        builder = builder.with_base(base);
    }

    let api = builder.into_openapi();
    let api_yaml = serde_yaml::to_string(&api)?;

    info!("Writing OpenAPI definition to {target}");
    DefaultFileSystem.write_file(&target, api_yaml)?;

    Ok(())
}

fn main() -> ExitCode {
    let config = match config::Config::new(None) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {}", err);
            return ExitCode::FAILURE;
        }
    };

    stderrlog::new()
        .quiet(config.is_quiet())
        .verbosity(config.verbosity())
        .timestamp(stderrlog::Timestamp::Off)
        .init()
        .unwrap();

    if let Err(err) = run(config) {
        error!("{}", err);
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
