use super::compile::compile;
use super::module::{Module, ModuleSet};
use crate::locator::Locator;

#[test]
fn compile_modules() -> anyhow::Result<()> {
    let input = std::fs::read_to_string("../examples/main.oal")?;
    let tree = oal_syntax::rewrite::parse(input)?;
    let base = Locator::try_from("file:///main.oal")?;
    let main = Module::new(base.clone(), tree);

    let mut mods = ModuleSet::new(main);

    let input = std::fs::read_to_string("../examples/module.oal")?;
    let tree = oal_syntax::rewrite::parse(input)?;
    let loc = Locator::try_from("file:///module.oal")?;
    let module = Module::new(loc.clone(), tree);
    mods.insert(module);

    compile(&mods, &loc)?;
    compile(&mods, &base)?;

    Ok(())
}
