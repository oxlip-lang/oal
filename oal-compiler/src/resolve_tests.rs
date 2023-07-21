use crate::definition::Definition;
use crate::errors::Kind;
use crate::module::ModuleSet;
use crate::resolve::resolve;
use crate::tests::mods_from;
use crate::tree::NRef;
use oal_model::grammar::AbstractSyntaxNode;
use oal_syntax::lexer as lex;
use oal_syntax::parser::{
    Application, Binding, Declaration, Primitive, Program, Terminal, Variable,
};

fn definition<'a>(mods: &'a ModuleSet, node: NRef<'a>) -> NRef<'a> {
    let core = node.syntax().core_ref();
    let defn = core.definition().expect("expected a definition");
    let Definition::External(ext) = defn else { panic!("expected an external") };
    ext.node(mods)
}

#[test]
fn resolve_variable() -> anyhow::Result<()> {
    let mods = mods_from(
        r#"
    let a = num;
    let b = a;
"#,
    )?;

    resolve(&mods, mods.base()).expect("expected resolution");

    let prog = Program::cast(mods.main().root()).expect("expected a program");

    let decl = prog.declarations().nth(1).expect("expected a declaration");

    let var = Variable::cast(
        Terminal::cast(decl.rhs())
            .expect("expected a terminal")
            .inner(),
    )
    .expect("expected a variable");

    let defn = definition(&mods, var.node());

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

    let prog = Program::cast(mods.main().root()).expect("expected a program");

    let decl = prog.declarations().nth(1).expect("expected a declaration");

    let app = Application::cast(decl.rhs()).expect("expected an application");

    let defn = definition(&mods, app.lambda().node());

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

    let defn = definition(&mods, var.node());

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

#[test]
fn resolve_recursion() -> anyhow::Result<()> {
    let mods = mods_from(
        r#"
    let a = { 'b b }; // mutually recursive
    let b = { 'a a }; // ...
    let c = { 'a a, 'b b }; // non-recursive
    let d = { 'd d }; // self-recursive
"#,
    )?;

    resolve(&mods, mods.base()).expect("expected resolution");

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
