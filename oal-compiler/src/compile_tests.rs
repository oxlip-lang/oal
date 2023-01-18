use crate::compile::compile;
use crate::module::ModuleSet;
use oal_model::locator::Locator;

#[test]
fn compile_modules() -> anyhow::Result<()> {
    let base = Locator::try_from("file:///main.oal")?;
    let input = std::fs::read_to_string("../examples/main.oal")?;
    let (main, errs) = oal_syntax::parse(base.clone(), input);
    assert!(errs.is_empty());
    let main = main.expect("parsing failed");

    let mut mods = ModuleSet::new(main);

    let loc = Locator::try_from("file:///module.oal")?;
    let input = std::fs::read_to_string("../examples/module.oal")?;
    let (module, errs) = oal_syntax::parse(loc.clone(), input);
    assert!(errs.is_empty());
    let module = module.expect("parsing failed");

    mods.insert(module);

    compile(&mods, &loc)?;
    compile(&mods, &base)?;

    Ok(())
}
