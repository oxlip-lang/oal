use crate::module::ModuleSet;
use oal_model::locator::Locator;

pub fn mods_from(code: &str) -> anyhow::Result<ModuleSet> {
    let loc = Locator::try_from("file:///base")?;
    let main = oal_syntax::parse(loc, code)?;
    Ok(ModuleSet::new(main))
}
