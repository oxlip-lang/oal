use crate::inference::{TagSeq, TypeConstraint};
use crate::scope::Env;
use crate::{constrain, substitute, tag_type, Scan, Transform};
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

    d.transform(seq, env, tag_type).expect("tagging failed");

    println!("{:#?}", d);

    if let Stmt::Decl(decl) = d.stmts.first().unwrap() {
        if Some(Tag::Primitive) != decl.expr.tag {
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

    d.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)
        .expect("tagging failed");

    println!("{:#?}", d);

    let cnt = &mut TypeConstraint::new();

    d.scan(cnt, &mut Env::new(), constrain)
        .expect("constraining failed");

    println!("{:#?}", cnt);

    let subst = &mut cnt.unify().expect("unification failed");

    println!("{:#?}", subst);

    let t = subst.substitute(Tag::Var(0));

    assert_eq!(t, Tag::Object);

    d.transform(subst, &mut Env::new(), substitute)
        .expect("substitution failed");

    println!("{:#?}", d);
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
