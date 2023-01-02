use crate::module::{Module, ModuleSet};
use oal_model::locator::Locator;

pub fn mods_from(code: &str) -> anyhow::Result<ModuleSet> {
    let tree = oal_syntax::parse(code)?;
    let loc = Locator::try_from("file:///base")?;
    let main = Module::new(loc, tree);
    Ok(ModuleSet::new(main))
}
