use crate::errors;
use crate::module::{load, ModuleSet};
use oal_model::locator::Locator;

#[test]
fn module_cycle() -> anyhow::Result<()> {
    let base = Locator::try_from("file::///base.oal")?;
    let module = Locator::try_from("file::///module.oal")?;

    let loader = |loc: Locator| {
        let code = if loc == base {
            r#"use "module.oal";"#
        } else if loc == module {
            r#"use "base.oal";"#
        } else {
            unreachable!()
        };
        let (tree, errs) = oal_syntax::parse(loc, code);
        assert!(errs.is_empty());
        let tree = tree.expect("parsing failed");
        Ok(tree)
    };

    let compiler = |_mods: &ModuleSet, _loc: &Locator| Ok(());

    let err: anyhow::Error = load(&base, loader, compiler).expect_err("expected an error");

    assert!(matches!(
        err.downcast_ref::<errors::Error>()
            .expect("expected compiler error")
            .kind,
        errors::Kind::CycleDetected
    ));

    Ok(())
}

#[test]
fn module_sort() -> anyhow::Result<()> {
    let base = Locator::try_from("file::///base.oal")?;
    let module1 = Locator::try_from("file::///module1.oal")?;
    let module2 = Locator::try_from("file::///module2.oal")?;

    let loader = |loc: Locator| {
        let code = if loc == base {
            r#"
            use "module2.oal";
            res a;
            "#
        } else if loc == module1 {
            r#"
            let a = /;
            "#
        } else if loc == module2 {
            r#"
            use "module1.oal";
            "#
        } else {
            unreachable!()
        };
        let (tree, errs) = oal_syntax::parse(loc, code);
        assert!(errs.is_empty());
        let tree = tree.expect("parsing failed");
        Ok(tree)
    };

    let mut order = Vec::new();

    let compiler = |_mods: &ModuleSet, loc: &Locator| {
        order.push(loc.clone());
        anyhow::Ok(())
    };

    let _mods = load(&base, loader, compiler).expect("loading failed");

    assert_eq!(order.len(), 3, "expected 3 compilation units");
    assert_eq!(order[0], module1, "expect module1 to be compiled first");
    assert_eq!(order[1], module2, "expect module1 to be compiled first");
    assert_eq!(order[2], base, "expect module1 to be compiled first");

    Ok(())
}
