use crate::ast::{
    Application, Expr, Lambda, Method, Operator, Prim, Prop, Stmt, Uri, UriSegment, VariadicOp,
};
use crate::parse;

#[test]
fn parse_variable_decl() {
    let d = parse("let a = num;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        assert_eq!(decl.name.as_ref(), "a");
        assert_eq!(decl.expr.inner, Expr::Prim(Prim::Num));
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
            assert_eq!(array.item.inner, Expr::Prim(Prim::Str));
        } else {
            panic!("expected array expression");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_root_uri() {
    let d = parse("let a = /;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        if let Expr::Uri(uri) = &decl.expr.inner {
            assert_eq!(uri.spec.len(), 1);
            assert_eq!(*uri.spec.first().unwrap(), UriSegment::root());
        } else {
            panic!("expected uri expression");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_template_uri() {
    let d = parse("let a = /x/{ y str }/z/;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        if let Expr::Uri(uri) = &decl.expr.inner {
            assert_eq!(uri.spec.len(), 3);
            assert_eq!(*uri.spec.get(0).unwrap(), UriSegment::Literal("x".into()));
            assert_eq!(*uri.spec.get(2).unwrap(), UriSegment::Literal("z".into()));
            if let UriSegment::Variable(Prop { key, val }) = uri.spec.get(1).unwrap() {
                assert_eq!(key.as_ref(), "y");
                assert_eq!(val.inner, Expr::Prim(Prim::Str));
            } else {
                panic!("expected uri segment variable");
            }
        } else {
            panic!("expected uri expression");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_composite_relation() {
    let code = r#"
        let a = / (
            patch, put : {} -> {},
            get             -> {}
        );
    "#;
    let d = parse(code.into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        if let Expr::Rel(rel) = &decl.expr.inner {
            assert!(rel.xfers[Method::Get].is_some());
            assert!(rel.xfers[Method::Patch].is_some());
            assert!(rel.xfers[Method::Put].is_some());
            assert_eq!(rel.xfers.values().filter(|x| x.is_some()).count(), 3);
        } else {
            panic!("expected relation expression");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_simple_relation() {
    let d = parse("let a = / ( put : {} -> {} );".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        if let Expr::Rel(rel) = &decl.expr.inner {
            assert_eq!(
                rel.uri.inner,
                Expr::Uri(Uri {
                    spec: vec![UriSegment::root()]
                })
            );
            if let Some(xfer) = &rel.xfers[Method::Put] {
                if let Some(domain) = &xfer.domain {
                    assert_eq!(domain.inner, Expr::Object(Default::default()));
                } else {
                    panic!("expected domain expression");
                }
                assert_eq!(xfer.range.inner, Expr::Object(Default::default()));
            } else {
                panic!("expected transfer on HTTP PUT");
            }
        } else {
            panic!("expected relation expression");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_any_type() {
    let d = parse("let a = {} ~ uri ~ bool;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        if let Expr::Op(VariadicOp {
            op: Operator::Any,
            exprs,
        }) = &decl.expr.inner
        {
            assert_eq!(exprs.len(), 3);
        } else {
            panic!("expected untyped alternative operation");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_application() {
    let d = parse("let a = f num {} uri;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        if let Expr::App(Application { name, args }) = &decl.expr.inner {
            assert_eq!(name.as_ref(), "f");
            assert_eq!(args.len(), 3);
        } else {
            panic!("expected function application");
        }
    }
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
            assert_eq!(body.inner, Expr::Prim(Prim::Num));
        } else {
            panic!("expected lambda expression");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_annotation() {
    let code = r#"
        # description: "some identifer", required: true
        let id = num;
        # description: "some record"
        let r = {};
        let a = /{ n id } ( put : r -> r );
    "#;
    let d = parse(code.into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 5);
}
