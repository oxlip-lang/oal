use crate::ast::{Stmt, TypeExpr, TypePrim, TypeTag};
use crate::parse;

#[test]
fn parse_untyped_decl() {
    let d = parse("let id1 = num".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        assert_eq!(decl.var.as_ref(), "id1");
        if let TypeTag::Var(n) = decl.tag {
            assert_eq!(n, 4);
        } else {
            panic!("expected variable type tag");
        }
        if TypeExpr::Prim(TypePrim::Num) != decl.expr {
            panic!("expected numeric type expression");
        }
    } else {
        panic!("expected declaration");
    }
}
