use crate::errors;
use crate::locator::Locator;
use crate::module::{load, ModuleSet};

#[test]
fn module_cycle() -> anyhow::Result<()> {
    let base = &Locator::try_from("file::///base.oal")?;
    let module = &Locator::try_from("file::///module.oal")?;

    let loader = |loc: &Locator| {
        let code = if loc == base {
            r#"use "module.oal";"#
        } else if loc == module {
            r#"use "base.oal";"#
        } else {
            unreachable!()
        };
        Ok(oal_syntax::parse(code)?)
    };

    let compiler = |_mods: &ModuleSet, _loc: &Locator| Ok(());

    let err: anyhow::Error = load(base, loader, compiler).expect_err("expected an error");

    assert!(matches!(
        err.downcast_ref::<errors::Error>()
            .expect("expected compiler error")
            .kind,
        errors::Kind::CycleDetected
    ));

    Ok(())
}
