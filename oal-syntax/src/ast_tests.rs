use crate::ast::{Expr, Lambda, Prim, Stmt};
use crate::parse;

#[test]
fn parse_variable_decl() {
    let d = parse("let a = num;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        assert_eq!(decl.name.as_ref(), "a");
        if Expr::Prim(Prim::Num) != decl.expr.inner {
            panic!("expected numeric type expression");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_assignment() {
    let d = parse("let a = b;".into()).expect("parsing failed");
    assert_eq!(d.stmts.len(), 1);
}

#[test]
fn parse_array() {
    let d = parse("let a = [str];".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        if let Expr::Array(array) = &decl.expr.inner {
            if Expr::Prim(Prim::Str) != array.item.inner {
                panic!("expected string type expression");
            }
        } else {
            panic!("expected array expression");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_any_type() {
    let d = parse("let a = {} ~ uri ~ bool;".into()).expect("parsing failed");
    assert_eq!(d.stmts.len(), 1);
}

#[test]
fn parse_application() {
    let d = parse("let a = f num {} uri;".into()).expect("parsing failed");
    assert_eq!(d.stmts.len(), 1);
}

#[test]
fn parse_lambda_decl() {
    let d = parse("let f x y z = num;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        assert_eq!(decl.name.as_ref(), "f");
        if let Expr::Lambda(Lambda {
            body,
            bindings: args,
        }) = &decl.expr.inner
        {
            let bindings: Vec<_> = args
                .iter()
                .filter_map(|a| match &a.inner {
                    Expr::Binding(b) => Some(b.as_ref()),
                    _ => None,
                })
                .collect();
            assert_eq!(bindings, vec!["x", "y", "z"]);
            if Expr::Prim(Prim::Num) != body.inner {
                panic!("expected numeric type expression");
            }
        } else {
            panic!("expected lambda expression");
        }
    } else {
        panic!("expected declaration");
    }
}
