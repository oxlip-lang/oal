use crate::{
    compile, constrain, substitute, tag_type, Env, Scan, TagSeq, Transform, TypeConstraint,
};
use oal_syntax::parse;

#[test]
#[ignore]
fn compile_application() {
    let code = r#"
        let f x = x | num;
        let a = f bool;
    "#;
    let mut doc = parse(code.into()).expect("parsing failed");

    doc.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)
        .expect("tagging failed");

    let cnt = &mut TypeConstraint::new();

    doc.scan(cnt, &mut Env::new(), constrain)
        .expect("constraining failed");

    let subst = &mut cnt.unify().expect("unification failed");

    doc.transform(subst, &mut Env::new(), substitute)
        .expect("substitution failed");

    doc.transform(&mut (), &mut Env::new(), compile)
        .expect("compilation failed");
}
