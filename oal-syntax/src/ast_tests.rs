use crate::ast::{Expr, Prim, Stmt, Tag};
use crate::parse;

#[test]
fn parse_untyped_decl() {
    let d = parse("let id1 = num".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        assert_eq!(decl.var.as_ref(), "id1");
        if Expr::Prim(Prim::Num) != decl.body.expr {
            panic!("expected numeric type expression");
        }
    } else {
        panic!("expected declaration");
    }
}
