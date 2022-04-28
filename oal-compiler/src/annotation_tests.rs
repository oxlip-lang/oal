use crate::annotation::{annotate, Annotated};
use crate::reduction::reduce;
use crate::scope::Env;
use crate::transform::Transform;
use crate::Program;
use oal_syntax::ast::{Expr, Statement, UriSegment};
use oal_syntax::parse;
use serde_yaml::Value;

#[test]
fn annotate_simple() {
    let code = r#"
        # description: "some identifier", required: true
        let id = num;
        # description: "some record"
        let r = {};
        res /{ n id } ( put : r -> r );
    "#;
    let mut prg: Program = parse(code.into()).expect("parsing failed");

    assert_eq!(prg.stmts.len(), 5);

    prg.transform(&mut None, &mut Env::new(), annotate)
        .expect("annotation failed");

    prg.transform(&mut (), &mut Env::new(), reduce)
        .expect("reduction failed");

    if let Statement::Res(res) = prg.stmts.iter().nth(4).unwrap() {
        if let Expr::Rel(rel) = res.rel.as_ref() {
            if let Expr::Uri(uri) = rel.uri.as_ref().as_ref() {
                if let UriSegment::Variable(p) = uri.spec.first().expect("expected URI segment") {
                    let ann = p.val.annotation().expect("expected annotation");
                    let desc = ann
                        .props
                        .get(&Value::String("description".to_owned()))
                        .expect("expected description property")
                        .as_str()
                        .expect("expected string");
                    let req = ann
                        .props
                        .get(&Value::String("required".to_owned()))
                        .expect("expected required property")
                        .as_bool()
                        .expect("expected boolean");
                    assert_eq!((desc, req), ("some identifier", true));
                } else {
                    panic!("expected URI segment variable");
                }
            } else {
                panic!("expected URI");
            }
        } else {
            panic!("expected relation");
        }
    } else {
        panic!("expected resource");
    }
}
