use crate::module::ModuleSet;
use oal_model::locator::Locator;

pub fn mods_from(code: &str) -> anyhow::Result<ModuleSet> {
    let loc = Locator::try_from("file:base")?;
    let (tree, errs) = oal_syntax::parse(loc, code);
    if !errs.is_empty() {
        for err in errs.into_iter() {
            println!("{err}");
        }
        panic!("parsing failed")
    }
    let tree = tree.expect("expected a syntax tree");
    Ok(ModuleSet::new(tree))
}
