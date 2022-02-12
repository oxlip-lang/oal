use crate::inference::{TagSeq, TypeConstrained, TypeConstraint, TypeTagged};
use crate::scope::Env;
use oal_syntax::ast::{Stmt, Tag};
use oal_syntax::parse;

#[test]
fn tag_decl() {
    let code = r#"
        let id1 = num
        let id2 = id1 | {}
    "#;
    let mut d = parse(code.into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 2);

    let seq = &mut TagSeq::new();
    let env = &mut Env::new();

    d.tag_type(seq, env).expect("tagging failed");

    println!("{:#?}", d);

    if let Stmt::Decl(decl) = d.stmts.first().unwrap() {
        if Some(Tag::Primitive) != decl.body.tag {
            panic!("expected primitive type tag");
        }
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn constraint() {
    let code = r#"
        let id1 = {} & {}
        let id2 = id1 | {}
    "#;
    let mut d = parse(code.into()).expect("parsing failed");

    assert_eq!(d.stmts.len(), 2);

    let seq = &mut TagSeq::new();
    let env = &mut Env::new();

    d.tag_type(seq, env).expect("tagging failed");

    println!("{:#?}", d);

    let cnt = &mut TypeConstraint::new();

    d.constrain(cnt);

    println!("{:#?}", cnt);

    let u = cnt.unify().expect("unification failed");

    println!("{:#?}", u);

    let t = u.substitute(Tag::Var(0));

    assert_eq!(t, Tag::Object);
}

#[test]
fn unify() {
    let mut c = TypeConstraint::new();

    c.push(Tag::Var(0), Tag::Primitive);
    c.push(Tag::Var(2), Tag::Var(1));
    c.push(Tag::Var(1), Tag::Var(0));

    let u = c.unify().expect("unification failed");

    println!("{:#?}", u);

    let t = u.substitute(Tag::Var(2));

    assert_eq!(t, Tag::Primitive);
}
