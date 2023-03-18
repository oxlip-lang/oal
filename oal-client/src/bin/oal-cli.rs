use anyhow::anyhow;
use oal_client::{config, report, write_file, Context};
use oal_model::span::Span;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let config = config::Config::new(None)?;
    let main = config.main()?;
    let target = config.target()?;
    let base = config.base()?;

    let mut ctx = Context::new(std::io::stderr());

    let mods = oal_compiler::module::load(&mut ctx, &main)?;

    eprintln!("Generating API definition");
    let spec = match oal_compiler::eval::eval(&mods, mods.base()) {
        Err(err) => {
            let span = match err.span() {
                Some(s) => s.clone(),
                None => Span::new(mods.base().clone(), 0..0),
            };
            report(ctx.console(), span, &err)?;
            Err(anyhow!("evaluation failed"))
        }
        Ok(spec) => Ok(spec),
    }?;

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
    write_file(&target, api_yaml)?;

    Ok(())
}
