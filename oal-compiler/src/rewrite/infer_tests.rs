use super::infer::{constrain, tag};
use super::module::ModuleSet;
use super::resolve::resolve;
use super::tests::mods_from;
use crate::inference::tag::Tag;
use crate::rewrite::infer::{check_tags, substitute};
use oal_syntax::rewrite::parser::{Application, Program, Terminal, Variable};

fn compile(code: &str) -> anyhow::Result<ModuleSet> {
    let mods = mods_from(code)?;
    resolve(&mods, mods.base())?;
    tag(&mods, mods.base())?;
    Ok(mods)
}

#[test]
fn infer_tag() -> anyhow::Result<()> {
    let mods = compile(
        r#"
        let f x = x;
        let b = f num;
    "#,
    )?;

    let prog = Program::cast(mods.main().tree().root()).expect("expected a program");

    let decl1 = prog.declarations().nth(0).expect("expected a declaration");
    let Tag::Var(_) = decl1.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let binding = decl1.bindings().next().expect("expected a binding");
    assert_eq!(binding.ident(), "x");
    let Tag::Var(_) = binding.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let term = Terminal::cast(decl1.rhs()).expect("expected a terminal");
    let Tag::Var(_) = term.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let variable = Variable::cast(term.inner()).expect("expected a variable");
    assert_eq!(variable.ident(), "x");
    let Tag::Var(_) = variable.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let decl2 = prog.declarations().nth(1).expect("expected a declaration");
    let Tag::Var(_) = decl2.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let app = Application::cast(decl2.rhs()).expect("expected an application");
    let Tag::Var(_) = app.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let arg = app.arguments().next().expect("expected an argument");
    let Tag::Var(_) = arg.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    assert_eq!(arg.inner().syntax().core_ref().unwrap_tag(), Tag::Primitive);

    Ok(())
}

#[test]
fn infer_set() -> anyhow::Result<()> {
    let mods = compile(
        r#"
        let f x = x;
        let g y = f y;
        let a = g num;
        let b = a;
    "#,
    )?;

    let eqs = constrain(&mods, mods.base())?;
    let set = eqs.unify()?;
    substitute(&mods, mods.base(), &set)?;
    check_tags(&mods, mods.base())?;

    let prog = Program::cast(mods.main().tree().root()).expect("expected a program");
    let decl = prog.declarations().last().expect("expected a declaration");
    assert_eq!(decl.node().syntax().core_ref().unwrap_tag(), Tag::Primitive);

    Ok(())
}
