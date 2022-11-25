use super::resolve::resolve;
use super::tests::mods_from;
use oal_syntax::rewrite::lexer as lex;
use oal_syntax::rewrite::parser::{Declaration, Primitive, Program, Terminal, Variable};

#[test]
fn resolve_simple() -> anyhow::Result<()> {
    let mods = mods_from(
        r#"
    let a = num;
    let b = a;
"#,
    )?;

    resolve(&mods).expect("expected resolution");

    let prog = Program::cast(mods.main().tree().root()).expect("expected a program");

    let decl = prog.declarations().nth(1).expect("expected a declaration");

    let var = Variable::cast(
        Terminal::cast(decl.rhs())
            .expect("expected a terminal")
            .inner(),
    )
    .expect("expected a variable");

    let defn = var
        .node()
        .syntax()
        .core_ref()
        .definition()
        .expect("expected a definition")
        .node(&mods);

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
