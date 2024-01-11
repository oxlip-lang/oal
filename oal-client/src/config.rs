use clap::Parser as ClapParser;
use oal_model::locator::Locator;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use url::Url;

/// Compiles an Oxlip program into an OpenAPI description in YAML.
#[derive(ClapParser, Debug)]
struct Args {
    /// The relative URL to the main program
    #[arg(short = 'm', long)]
    main: Option<String>,

    /// The relative URL to the target OpenAPI description
    #[arg(short = 't', long)]
    target: Option<String>,

    /// The relative URL to a base OpenAPI description
    #[arg(short = 'b', long)]
    base: Option<String>,

    /// The path to the configuration file
    #[arg(short = 'c', long = "conf")]
    config: Option<PathBuf>,

    /// Increase message verbosity
    #[arg(short = 'v', long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Silence all output
    #[arg(short = 'q', long, conflicts_with = "verbose")]
    quiet: bool,
}

#[derive(Deserialize, Default, Debug)]
struct File {
    api: Api,
}

#[derive(Deserialize, Default, Debug)]
struct Api {
    main: Option<String>,
    target: Option<String>,
    base: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    args: Args,
    file: File,
    root: Locator,
}

fn path_locator(p: &Path) -> anyhow::Result<Locator> {
    let path = p.canonicalize()?;
    let url = Url::from_file_path(path).expect("absolute path should convert to URL");
    Ok(Locator::from(url))
}

impl Config {
    pub fn new(cfg: Option<&Path>) -> anyhow::Result<Self> {
        let args: Args = Args::parse();

        let config = cfg.or(args.config.as_deref());

        let (root, file) = if let Some(path) = config {
            let root = path_locator(path)?;
            let cfg = std::fs::read_to_string(path)?;
            let file = toml::from_str::<File>(&cfg)?;
            (root, file)
        } else {
            let cwd = std::env::current_dir()?;
            let loc = path_locator(cwd.as_path())?;
            let root = loc.as_base();
            let file = File::default();
            (root, file)
        };

        Ok(Config { args, file, root })
    }

    pub fn main(&self) -> anyhow::Result<Locator> {
        match self.args.main.as_ref().or(self.file.api.main.as_ref()) {
            Some(p) => Ok(self.root.join(p)?),
            None => Err(anyhow::Error::msg("main module not specified")),
        }
    }

    pub fn target(&self) -> anyhow::Result<Locator> {
        match self.args.target.as_ref().or(self.file.api.target.as_ref()) {
            Some(p) => Ok(self.root.join(p)?),
            None => Err(anyhow::Error::msg("target not specified")),
        }
    }

    pub fn base(&self) -> anyhow::Result<Option<Locator>> {
        match self.args.base.as_ref().or(self.file.api.base.as_ref()) {
            Some(p) => Ok(Some(self.root.join(p)?)),
            None => Ok(None),
        }
    }

    pub fn is_quiet(&self) -> bool {
        self.args.quiet
    }

    pub fn verbosity(&self) -> usize {
        self.args.verbose as usize
    }
}
