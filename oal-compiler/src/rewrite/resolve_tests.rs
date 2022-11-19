use super::{
    module::{Module, ModuleSet},
    resolve::resolve,
};
use crate::Locator;

#[test]
fn resolve_simple() -> anyhow::Result<()> {
    let code = r#"
        let a = num;
        let b = a;
    "#;
    let tree = oal_syntax::rewrite::parse(code)?;
    let loc = Locator::try_from("file:///base")?;
    let base = Module::new(loc, tree);
    let mods = &mut ModuleSet::new(base);
    resolve(&mods).expect("expected resolution");

    // TODO: check result of resolution

    Ok(())
}
