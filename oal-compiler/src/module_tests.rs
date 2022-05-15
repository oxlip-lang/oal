use crate::errors::{Kind, Result};
use crate::module::load;
use crate::{ModuleSet, Program};
use oal_syntax::ast::Locator;
use oal_syntax::parse;
use std::path::Path;

#[test]
fn module_simple() {
    let loc = &Locator::from(Path::new("module.oal"));
    let loader = |l: &Locator| -> Result<Program> {
        assert_eq!(*l, *loc);
        Ok(parse("let id = num;").expect("parsing failed"))
    };
    let compiler = |_mods: &ModuleSet, _l: &Locator, p: Program| -> Result<Program> { Ok(p) };
    let mods = load(loc, loader, compiler).expect("loading failed");

    assert_eq!(mods.len(), 1);
    assert_eq!(*mods.keys().next().unwrap(), *loc);
}

#[test]
fn module_cycle() {
    let loc = &Locator::from(Path::new("module.oal"));
    let loader = |l: &Locator| -> Result<Program> {
        assert_eq!(*l, *loc);
        Ok(parse(r#"use "module.oal";"#).expect("parsing failed"))
    };
    let compiler = |_mods: &ModuleSet, _l: &Locator, p: Program| -> Result<Program> { Ok(p) };
    assert_eq!(
        load(loc, loader, compiler)
            .expect_err("expected cycle")
            .kind,
        Kind::CycleDetected
    );
}
