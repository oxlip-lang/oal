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

    d.tag_type(seq, env);

    println!("{:#?}", d);

    if let Stmt::Decl(decl) = d.stmts.first().unwrap() {
        if Some(Tag::Number) != decl.body.tag {
            panic!("expected numeric type tag");
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

    d.tag_type(seq, env);

    println!("{:#?}", d);

    let cnt = &mut TypeConstraint::new();

    d.constrain(cnt);

    println!("{:#?}", cnt);
}
