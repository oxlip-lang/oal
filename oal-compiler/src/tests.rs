use crate::module::ModuleSet;
use oal_model::locator::Locator;

pub fn mods_from(code: &str) -> anyhow::Result<ModuleSet> {
    let loc = Locator::try_from("file:///base")?;
    let (tree, errs) = oal_syntax::parse(loc, code);
    assert!(errs.is_empty());
    let tree = tree.expect("parsing failed");
    Ok(ModuleSet::new(tree))
}
