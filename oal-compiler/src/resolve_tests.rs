use crate::errors::Kind;
use crate::resolve::resolve;
use crate::tests::mods_from;
use crate::tree::definition;
use oal_syntax::lexer as lex;
use oal_syntax::parser::{
    Application, Binding, Declaration, Primitive, Program, Terminal, Variable,
};

#[test]
fn resolve_variable() -> anyhow::Result<()> {
    let mods = mods_from(
        r#"
    let a = num;
    let b = a;
"#,
    )?;

    resolve(&mods, mods.base()).expect("expected resolution");

    let prog = Program::cast(mods.main().tree().root()).expect("expected a program");

    let decl = prog.declarations().nth(1).expect("expected a declaration");

    let var = Variable::cast(
        Terminal::cast(decl.rhs())
            .expect("expected a terminal")
            .inner(),
    )
    .expect("expected a variable");

    let defn = definition(&mods, var.node()).expect("expected a definition");

    let decl = Declaration::cast(defn).expect("expected a declaration");

    assert_eq!(
        Primitive::cast(
            Terminal::cast(decl.rhs())
                .expect("expected a terminal")
                .inner()
        )
        .expect("expected a primitive")
        .primitive(),
        lex::Primitive::Num
    );

    Ok(())
}

#[test]
fn resolve_application() -> anyhow::Result<()> {
    let mods = mods_from(
        r#"
    let f x = x;
    let b = f num;
"#,
    )?;

    resolve(&mods, mods.base()).expect("expected resolution");

    let prog = Program::cast(mods.main().tree().root()).expect("expected a program");

    let decl = prog.declarations().nth(1).expect("expected a declaration");

    let app = Application::cast(decl.rhs()).expect("expected an application");

    let defn = definition(&mods, app.lambda().node()).expect("expected a definition");

    let decl = Declaration::cast(defn).expect("expected a declaration");

    let bindings: Vec<_> = decl.bindings().map(|i| i.ident().to_string()).collect();

    assert_eq!(bindings, vec!["x"]);

    let var = Variable::cast(
        Terminal::cast(decl.rhs())
            .expect("expected a terminal")
            .inner(),
    )
    .expect("expected a variable");

    assert_eq!(var.ident(), "x");

    let defn = definition(&mods, var.node()).expect("expected a definition");

    let binding = Binding::cast(defn).expect("expected a binding");

    assert_eq!(binding.ident(), "x");

    Ok(())
}

#[test]
fn resolve_not_in_scope() -> anyhow::Result<()> {
    let mods = mods_from("let a = f {};")?;

    if let Err(e) = resolve(&mods, mods.base()) {
        assert!(matches!(e.kind, Kind::NotInScope));
    } else {
        panic!("expected an error");
    }

    Ok(())
}
