use super::{
    module::{Module, ModuleSet},
    resolve::resolve,
};
use crate::Locator;
use oal_syntax::rewrite::lexer as lex;
use oal_syntax::rewrite::parser::{Primitive, Program, Terminal, Variable};

#[test]
fn resolve_simple() -> anyhow::Result<()> {
    let code = r#"
        let a = num;
        let b = a;
    "#;
    
    let tree = oal_syntax::rewrite::parse(code)?;
    let loc = Locator::try_from("file:///base")?;
    let main = Module::new(loc, tree);
    let mods = &mut ModuleSet::new(main);

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
        .node(mods);

    assert_eq!(
        Primitive::cast(Terminal::cast(defn).expect("expected a terminal").inner())
            .expect("expected a primitive")
            .primitive(),
        lex::Primitive::Num
    );

    Ok(())
}
