use crate::rewrite::lexer as lex;
use crate::rewrite::parser::{Gram, Primitive, Program, Terminal};
use oal_model::grammar::SyntaxTree;

fn parse(i: &str) -> SyntaxTree<(), Gram> {
    crate::rewrite::parse(i).expect("parsing failed")
}

#[test]
fn parse_variable_decl() {
    let tree = parse("let a = num;");

    let prog = Program::cast(tree.root()).unwrap();

    assert_eq!(prog.declarations().count(), 1, "expected one declaration");

    let decl = prog.declarations().next().unwrap();

    assert_eq!(decl.symbol().ident().as_ref(), "a");

    let term = Terminal::cast(decl.rhs()).expect("expected a terminal");
    let prim = Primitive::cast(term.inner()).expect("expected a primitive");

    assert_eq!(prim.kind(), lex::Primitive::Num, "expected a numeric type");
}
