use crate::ast::*;
use crate::parse;

#[derive(Clone, Debug, PartialEq)]
struct TestExpr(Expr<TestExpr>);

impl From<Expr<TestExpr>> for TestExpr {
    fn from(e: Expr<TestExpr>) -> Self {
        TestExpr(e)
    }
}

impl AsRef<Expr<TestExpr>> for TestExpr {
    fn as_ref(&self) -> &Expr<TestExpr> {
        &self.0
    }
}

impl AsMut<Expr<TestExpr>> for TestExpr {
    fn as_mut(&mut self) -> &mut Expr<TestExpr> {
        &mut self.0
    }
}

type Program = crate::ast::Program<TestExpr>;

#[test]
fn parse_variable_decl() {
    let d: Program = parse("let a = num;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Statement::Decl(decl) = s {
        assert_eq!(decl.name.as_ref(), "a");
        assert_eq!(*decl.expr.as_ref(), Expr::Prim(Primitive::Num));
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_assignment() {
    let d: Program = parse("let a = b;".into()).expect("parsing failed");
    assert_eq!(d.stmts.len(), 1);
}

#[test]
fn parse_array() {
    let d: Program = parse("let a = [str];".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Statement::Decl(decl) = s {
        if let Expr::Array(array) = decl.expr.as_ref() {
            assert_eq!(*array.item.as_ref().as_ref(), Expr::Prim(Primitive::Str));
        } else {
            panic!("expected array expression");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn parse_root_uri() {
    let d: Program = parse("let a = /;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Statement::Decl(decl) = s {
        if let Expr::Uri(uri) = decl.expr.as_ref() {
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
    let d: Program = parse("let a = /x/{ y str }/z/;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Statement::Decl(decl) = s {
        if let Expr::Uri(uri) = decl.expr.as_ref() {
            assert_eq!(uri.spec.len(), 3);
            assert_eq!(*uri.spec.get(0).unwrap(), UriSegment::Literal("x".into()));
            assert_eq!(*uri.spec.get(2).unwrap(), UriSegment::Literal("z".into()));
            if let UriSegment::Variable(Property { key, val }) = uri.spec.get(1).unwrap() {
                assert_eq!(key.as_ref(), "y");
                assert_eq!(*val.as_ref(), Expr::Prim(Primitive::Str));
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
    let d: Program = parse(code.into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Statement::Decl(decl) = s {
        if let Expr::Rel(rel) = decl.expr.as_ref() {
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
    let d: Program = parse("let a = / ( put : {} -> {} );".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Statement::Decl(decl) = s {
        if let Expr::Rel(rel) = decl.expr.as_ref() {
            assert_eq!(
                *rel.uri.as_ref().as_ref(),
                Expr::Uri(Uri {
                    spec: vec![UriSegment::root()]
                })
            );
            if let Some(xfer) = &rel.xfers[Method::Put] {
                if let Some(domain) = &xfer.domain {
                    assert_eq!(*domain.as_ref().as_ref(), Expr::Object(Default::default()));
                } else {
                    panic!("expected domain expression");
                }
                assert_eq!(
                    *xfer.range.as_ref().as_ref(),
                    Expr::Object(Default::default())
                );
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
    let d: Program = parse("let a = {} ~ uri ~ bool;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Statement::Decl(decl) = s {
        if let Expr::Op(VariadicOp {
            op: Operator::Any,
            exprs,
        }) = decl.expr.as_ref()
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
    let d: Program = parse("let a = f num {} uri;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Statement::Decl(decl) = s {
        if let Expr::App(Application { name, args }) = decl.expr.as_ref() {
            assert_eq!(name.as_ref(), "f");
            assert_eq!(args.len(), 3);
        } else {
            panic!("expected function application");
        }
    }
}

#[test]
fn parse_lambda_decl() {
    let d: Program = parse("let f x y z = num;".into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Statement::Decl(decl) = s {
        assert_eq!(decl.name.as_ref(), "f");
        if let Expr::Lambda(Lambda {
            body,
            bindings: args,
        }) = decl.expr.as_ref()
        {
            let bindings: Vec<_> = args
                .iter()
                .filter_map(|a| match a.as_ref() {
                    Expr::Binding(b) => Some(b.as_ref()),
                    _ => None,
                })
                .collect();
            assert_eq!(bindings, vec!["x", "y", "z"]);
            assert_eq!(*body.as_ref().as_ref(), Expr::Prim(Primitive::Num));
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
        # description: "some identifer"
        let id = num;
        # description: "some record"
        let r = {};
        let a = /{ n id } ( put : r -> r );
    "#;
    let d: Program = parse(code.into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 5);
}
