use super::resolve::resolve;
use super::tests::mods_from;
use super::tree::definition;
use oal_syntax::rewrite::lexer as lex;
use oal_syntax::rewrite::parser::{
    Application, Declaration, Identifier, Primitive, Program, Terminal, Variable,
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

    let defn = definition(&mods, app.node()).expect("expected a definition");

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

    let binding = Identifier::cast(defn).expect("expected an identifier");

    assert_eq!(binding.ident(), "x");

    Ok(())
}
