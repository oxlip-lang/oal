use crate::rewrite::lexer as lex;
use crate::rewrite::parser::{Declaration, Gram, Primitive, Program, Symbol, Terminal};
use oal_model::grammar::NodeRef;

fn parse<F: Fn(Program<()>)>(i: &str, f: F) {
    let tree = crate::rewrite::parse(i).expect("parsing failed");
    let prog = Program::cast(tree.root()).expect("expected a program");
    f(prog)
}

fn assert_decl<'a>(p: Program<'a, ()>, sym: &str) -> Declaration<'a, ()> {
    assert_eq!(p.declarations().count(), 1, "expected one declaration");
    let d = p.declarations().next().unwrap();
    assert_eq!(d.symbol().ident().as_ref(), sym);
    d
}

fn assert_term(n: NodeRef<(), Gram>) -> NodeRef<(), Gram> {
    let term = Terminal::cast(n).expect("expected a terminal");
    term.inner()
}

#[test]
fn parse_variable_decl() {
    parse("let a = num;", |p: Program<()>| {
        let decl = assert_decl(p, "a");
        let rhs = assert_term(decl.rhs());
        let prim = Primitive::cast(rhs).expect("expected a primitive");
        assert_eq!(
            prim.primitive(),
            lex::Primitive::Num,
            "expected a numeric type"
        );
    });
    parse("let a = b;", |p: Program<()>| {
        let decl = assert_decl(p, "a");
        let rhs = assert_term(decl.rhs());
        let sym = Symbol::cast(rhs).expect("expected a symbol");
        assert_eq!(sym.ident().as_ref(), "b");
    });
}
