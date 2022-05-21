use crate::errors::{Kind, Result};
use crate::locator::Locator;
use crate::module::load;
use crate::{ModuleSet, Program};
use oal_syntax::parse;

#[test]
fn module_simple() {
    let loc = &Locator::try_from("test:module.oal").unwrap();
    let loader = |l: &Locator| -> Result<Program> {
        assert_eq!(*l, *loc);
        Ok(parse("let id = num;").expect("parsing failed"))
    };
    let compiler = |_mods: &ModuleSet, _l: &Locator, p: Program| -> Result<Program> { Ok(p) };
    let mods = load(loc, loader, compiler).expect("loading failed");

    assert_eq!(mods.programs.len(), 1);
    assert_eq!(*mods.programs.keys().next().unwrap(), *loc);
}

#[test]
fn module_cycle() {
    let loc = &Locator::try_from("test:module.oal").unwrap();
    let loader = |l: &Locator| -> Result<Program> {
        assert_eq!(*l, *loc);
        Ok(parse(r#"use "test:module.oal";"#).expect("parsing failed"))
    };
    let compiler = |_mods: &ModuleSet, _l: &Locator, p: Program| -> Result<Program> { Ok(p) };
    assert_eq!(
        load(loc, loader, compiler)
            .expect_err("expected cycle")
            .kind,
        Kind::CycleDetected
    );
}
