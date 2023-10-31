use crate::errors::{Error, Kind};
use crate::module::{load, Loader, ModuleSet};
use crate::tree::Tree;
use oal_model::locator::Locator;
use std::cell::RefCell;

struct ContextCycle {
    base: Locator,
    module: Locator,
}

impl Loader<anyhow::Error> for ContextCycle {
    fn is_valid(&mut self, loc: &Locator) -> bool {
        let s = loc.url().as_str();
        s == "file:///module.oal" || s == "file:///base.oal"
    }

    fn load(&mut self, loc: &Locator) -> anyhow::Result<String> {
        let code = if *loc == self.base {
            r#"use "module.oal";"#
        } else if *loc == self.module {
            r#"use "base.oal";"#
        } else {
            unreachable!()
        };
        Ok(code.to_owned())
    }

    fn parse(&mut self, loc: Locator, input: String) -> anyhow::Result<Tree> {
        let (tree, errs) = oal_syntax::parse(loc, input);
        assert!(errs.is_empty());
        let tree = tree.expect("parsing failed");
        Ok(tree)
    }

    fn compile(&mut self, _mods: &ModuleSet, _loc: &Locator) -> anyhow::Result<()> {
        Ok(())
    }
}

#[test]
fn module_cycle() -> anyhow::Result<()> {
    let base = Locator::try_from("file:base.oal")?;
    let module = Locator::try_from("file:module.oal")?;
    let mut ctx = ContextCycle {
        base: base.clone(),
        module,
    };

    let err = load(&mut ctx, &base).expect_err("expected an error");

    assert!(matches!(
        err.downcast_ref::<Error>()
            .expect("expected compiler error")
            .kind,
        Kind::CycleDetected
    ));

    Ok(())
}

struct ContextSort {
    base: Locator,
    module1: Locator,
    module2: Locator,
    order: RefCell<Vec<Locator>>,
}

impl Loader<anyhow::Error> for ContextSort {
    fn is_valid(&mut self, loc: &Locator) -> bool {
        let s = loc.url().as_str();
        s == "file:///module1.oal" || s == "file:///module2.oal"
    }

    fn load(&mut self, loc: &Locator) -> anyhow::Result<String> {
        let code = if *loc == self.base {
            r#"
            use "module2.oal" as mod;
            res mod.a;
            "#
        } else if *loc == self.module1 {
            r#"
            let a = /;
            "#
        } else if *loc == self.module2 {
            r#"
            use "module1.oal";
            "#
        } else {
            unreachable!()
        };
        Ok(code.to_owned())
    }

    fn parse(&mut self, loc: Locator, input: String) -> anyhow::Result<Tree> {
        let (tree, errs) = oal_syntax::parse(loc, input);
        assert!(errs.is_empty());
        let tree = tree.expect("parsing failed");
        Ok(tree)
    }

    fn compile(&mut self, _mods: &ModuleSet, loc: &Locator) -> anyhow::Result<()> {
        self.order.borrow_mut().push(loc.clone());
        Ok(())
    }
}

#[test]
fn module_sort() -> anyhow::Result<()> {
    let base = Locator::try_from("file:base.oal")?;
    let module1 = Locator::try_from("file:module1.oal")?;
    let module2 = Locator::try_from("file:module2.oal")?;

    let mut ctx = ContextSort {
        base: base.clone(),
        module1: module1.clone(),
        module2: module2.clone(),
        order: Default::default(),
    };

    let _mods = load(&mut ctx, &base).expect("loading failed");

    let order = ctx.order.borrow();

    assert_eq!(order.len(), 3, "expected 3 compilation units");
    assert_eq!(order[0], module1, "expect module1 to be compiled first");
    assert_eq!(order[1], module2, "expect module2 to be compiled second");
    assert_eq!(order[2], base, "expect base to be compiled last");

    Ok(())
}

struct ContextInvalid;

impl Loader<anyhow::Error> for ContextInvalid {
    fn is_valid(&mut self, loc: &Locator) -> bool {
        assert_eq!(loc.url().as_str(), "file:///invalid.oal");
        false
    }

    fn load(&mut self, _loc: &Locator) -> anyhow::Result<String> {
        let code = r#"
            use "invalid.oal";
        "#;
        Ok(code.to_owned())
    }

    fn parse(&mut self, loc: Locator, input: String) -> anyhow::Result<Tree> {
        let (tree, errs) = oal_syntax::parse(loc, input);
        assert!(errs.is_empty());
        let tree = tree.expect("parsing failed");
        Ok(tree)
    }

    fn compile(&mut self, _mods: &ModuleSet, _loc: &Locator) -> anyhow::Result<()> {
        Ok(())
    }
}

#[test]
fn module_invalid() -> anyhow::Result<()> {
    let base = Locator::try_from("file:base.oal")?;

    let mut ctx = ContextInvalid;

    let err = load(&mut ctx, &base).expect_err("expected an error");

    assert!(matches!(
        err.downcast_ref::<Error>()
            .expect("expected compiler error")
            .kind,
        Kind::InvalidModule(_)
    ));

    Ok(())
}
