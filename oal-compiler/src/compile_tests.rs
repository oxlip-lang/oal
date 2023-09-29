use crate::compile::compile;
use crate::module::ModuleSet;
use crate::tests::mods_from;
use oal_model::grammar::AbstractSyntaxNode;
use oal_model::locator::Locator;
use oal_syntax::parser::Program;

#[test]
fn compile_modules() -> anyhow::Result<()> {
    let base = Locator::try_from("file:main.oal")?;
    let input = std::fs::read_to_string("../examples/main.oal")?;
    let (main, errs) = oal_syntax::parse(base.clone(), input);
    assert!(errs.is_empty());
    let main = main.expect("parsing failed");

    let mut mods = ModuleSet::new(main);

    let loc = Locator::try_from("file:module.oal")?;
    let input = std::fs::read_to_string("../examples/module.oal")?;
    let (module, errs) = oal_syntax::parse(loc.clone(), input);
    assert!(errs.is_empty());
    let module = module.expect("parsing failed");

    mods.insert(module);

    compile(&mods, &loc)?;
    compile(&mods, &base)?;

    Ok(())
}

#[test]
fn compile_cycles() -> anyhow::Result<()> {
    let mods = mods_from(
        r#"
    let a = { 'b b }; // mutually recursive
    let b = { 'a a }; // ...
    let c = { 'a a, 'b b }; // non-recursive
    let d = { 'd d }; // self-recursive
"#,
    )?;

    compile(&mods, mods.base()).expect("should compile");

    let prog = Program::cast(mods.main().root()).expect("expected a program");
    let a = prog.declarations().nth(0).expect("expected a declaration");
    let b = prog.declarations().nth(1).expect("expected a declaration");
    let c = prog.declarations().nth(2).expect("expected a declaration");
    let d = prog.declarations().nth(3).expect("expected a declaration");

    assert!(a.node().syntax().core_ref().is_recursive);
    assert!(b.node().syntax().core_ref().is_recursive);
    assert!(!c.node().syntax().core_ref().is_recursive);
    assert!(d.node().syntax().core_ref().is_recursive);

    Ok(())
}
