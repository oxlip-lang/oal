use crate::inference::{TagSeq, TypeConstraint};
use crate::scope::Env;
use crate::{constrain, tag_type, Scan, Transform};
use oal_syntax::ast::{Expr, Lambda, Stmt, Tag};
use oal_syntax::parse;

#[test]
fn tag_var_decl() {
    let code = r#"
        let id1 = num
    "#;
    let mut d = parse(code.into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 1);

    d.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)
        .expect("tagging failed");

    if let Stmt::Decl(decl) = d.stmts.first().unwrap() {
        if Some(Tag::Primitive) != decl.expr.tag {
            panic!("expected primitive type tag");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn tag_lambda_decl() {
    let mut d = parse("let f x y z = num".into()).expect("parsing failed");

    d.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)
        .expect("tagging failed");

    assert_eq!(d.stmts.len(), 1);

    let s = d.stmts.first().unwrap();

    if let Stmt::Decl(decl) = s {
        assert_eq!(decl.name.as_ref(), "f");
        assert_eq!(decl.expr.tag, Some(Tag::Var(0)));
        if let Expr::Lambda(Lambda { bindings, .. }) = &decl.expr.inner {
            let tags: Vec<_> = bindings
                .iter()
                .filter_map(|a| match a.tag {
                    Some(Tag::Var(n)) => Some(n),
                    _ => None,
                })
                .collect();
            assert_eq!(tags, vec![1, 2, 3]);
        } else {
            panic!("expected lambda expression");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn constraint_var() {
    let code = r#"
        let id1 = {} & {}
        let id2 = id1 | {}
    "#;
    let mut d = parse(code.into()).expect("parsing failed");

    d.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)
        .expect("tagging failed");

    let cnt = &mut TypeConstraint::new();

    d.scan(cnt, &mut Env::new(), constrain)
        .expect("constraining failed");

    assert_eq!(cnt.len(), 8);
}

#[test]
fn constraint_lambda() {
    let mut d = parse("let f x y z = num".into()).expect("parsing failed");

    d.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)
        .expect("tagging failed");

    let cnt = &mut TypeConstraint::new();

    d.scan(cnt, &mut Env::new(), constrain)
        .expect("constraining failed");

    assert_eq!(cnt.len(), 2);
}

#[test]
fn unify_simple() {
    let mut c = TypeConstraint::new();

    c.push(Tag::Var(0), Tag::Primitive);
    c.push(Tag::Var(2), Tag::Var(1));
    c.push(Tag::Var(1), Tag::Var(0));

    let u = c.unify().expect("unification failed");

    let t = u.substitute(&Tag::Var(2));

    assert_eq!(t, Tag::Primitive);
}

#[test]
fn unify_lambda() {
    let code = r#"
        let f x y z = num
        let a = f num {} uri
    "#;
    let mut d = parse(code.into()).expect("parsing failed");

    d.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)
        .expect("tagging failed");

    println!("{:#?}", d);

    let cnt = &mut TypeConstraint::new();

    d.scan(cnt, &mut Env::new(), constrain)
        .expect("constraining failed");

    println!("{:#?}", cnt);
}
