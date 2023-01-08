use super::{check_complete, constrain, substitute, tag};
use crate::inference::tag::Tag;
use crate::module::ModuleSet;
use crate::resolve::resolve;
use crate::tests::mods_from;
use oal_syntax::parser::{Application, Program, Terminal, Variable};

fn compile(code: &str) -> anyhow::Result<(ModuleSet, usize)> {
    let mods = mods_from(code)?;
    resolve(&mods, mods.base())?;
    let nvars = tag(&mods, mods.base())?;
    Ok((mods, nvars))
}

#[test]
fn infer_tag() -> anyhow::Result<()> {
    let (mods, _) = compile(
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

    let term = Terminal::cast(decl1.rhs()).expect("expected a terminal");
    let Tag::Var(t3) = term.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let variable = Variable::cast(term.inner()).expect("expected a variable");
    assert_eq!(variable.ident(), "x");
    let Tag::Var(t4) = variable.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let decl2 = prog.declarations().nth(1).expect("expected a declaration");
    let Tag::Var(t5) = decl2.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let app = Application::cast(decl2.rhs()).expect("expected an application");
    let Tag::Var(t6) = app.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    let arg = app.arguments().next().expect("expected an argument");
    let Tag::Var(t7) = arg.node().syntax().core_ref().unwrap_tag()
        else { panic!("expected a tag variable") };

    assert_eq!(arg.inner().syntax().core_ref().unwrap_tag(), Tag::Primitive);

    let mut vars = vec![t1, t2, t3, t4, t5, t6, t7];
    vars.sort();
    vars.dedup();
    assert_eq!(vars.len(), 7, "expected unique tag variables");

    Ok(())
}

#[test]
fn infer_unify() -> anyhow::Result<()> {
    let (mods, _nvars) = compile(
        r#"
        let f x = 'n x;
        let g y = f [y];
        let a = g num;
        let b = a;
    "#,
    )?;

    let eqs = constrain(&mods, mods.base())?;

    assert_eq!(eqs.len(), 18);

    let set = eqs.unify()?;
    substitute(&mods, mods.base(), &set)?;
    check_complete(&mods, mods.base())?;

    let prog = Program::cast(mods.main().tree().root()).expect("expected a program");
    let decl = prog.declarations().last().expect("expected a declaration");
    let tag = decl.node().syntax().core_ref().unwrap_tag();
    assert_eq!(tag, Tag::Property(Tag::Array.into()));

    Ok(())
}
