use super::infer::tag;
use super::module::ModuleSet;
use super::resolve::resolve;
use super::tests::mods_from;
use crate::inference::tag::Tag;
use oal_syntax::rewrite::parser::{Application, Program, Terminal, Variable};

fn infer(code: &str) -> anyhow::Result<ModuleSet> {
    let mods = mods_from(code)?;
    resolve(&mods, mods.base())?;
    tag(&mods, mods.base())?;
    Ok(mods)
}

#[test]
fn infer_tag() -> anyhow::Result<()> {
    let mods = infer(
        r#"
        let f x = x;
        let b = f num;
    "#,
    )?;

    let prog = Program::cast(mods.main().tree().root()).expect("expected a program");

    let decl1 = prog.declarations().nth(0).expect("expected a declaration");
    let Tag::Var(t1) = decl1.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let binding = decl1.bindings().next().expect("expected a binding");
    assert_eq!(binding.ident(), "x");
    let Tag::Var(t2) = binding.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let variable = Variable::cast(
        Terminal::cast(decl1.rhs())
            .expect("expected a terminal")
            .inner(),
    )
    .expect("expected a variable");
    assert_eq!(variable.ident(), "x");
    let Tag::Var(t3) = variable.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let decl2 = prog.declarations().nth(1).expect("expected a declaration");
    let Tag::Var(t4) = decl2.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let app = Application::cast(decl2.rhs()).expect("expected an application");
    let Tag::Var(t5) = app.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    assert_eq!(t1, t5, "application tag should equal function declaration");
    assert_eq!(t2, t3, "variable tag should equal binding");
    assert_ne!(t4, t1);
    assert_ne!(t4, t2);

    Ok(())
}
