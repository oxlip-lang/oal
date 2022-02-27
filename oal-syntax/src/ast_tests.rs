use crate::ast::{Expr, Prim, Stmt};
use crate::parse;

#[test]
fn parse_variable_decl() {
    let d = parse("let id1 = num".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        assert_eq!(decl.name.as_ref(), "id1");
        if Expr::Prim(Prim::Num) != decl.expr.expr {
            panic!("expected numeric type expression");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_any_type() {
    let d = parse("let id1 = num ~ {}".into()).expect("parsing failed");
    assert_eq!(d.stmts.len(), 1);
}

#[test]
fn parse_lambda_decl() {
    let d = parse("let f x y z = num".into()).expect("parsing failed");
    assert_eq!(d.stmts.len(), 1);
}
