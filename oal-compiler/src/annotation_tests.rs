use crate::annotation::{annotate, Annotated};
use crate::reduction::reduce;
use crate::scope::Env;
use crate::transform::Transform;
use crate::Program;
use oal_syntax::ast::{AsRefNode, Expr, Statement, UriSegment};
use oal_syntax::parse;

fn eval(code: &str) -> anyhow::Result<Program> {
    let mut prg: Program = parse(code)?;
    prg.transform(&mut None, &mut Env::new(None), &mut annotate)?;
    prg.transform(&mut (), &mut Env::new(None), &mut reduce)?;
    Ok(prg)
}

#[test]
fn annotate_simple() -> anyhow::Result<()> {
    let code = r#"
        # description: "some identifier", required: true
        let id = num | str;
        # description: "some record"
        let r = {};
        res /{ n id } ( put : <r> -> <r> );
    "#;
    let prg = eval(code)?;

    assert_eq!(prg.stmts.len(), 5);

    if let Statement::Res(res) = prg.stmts.iter().nth(4).unwrap() {
        if let Expr::Rel(rel) = res.rel.as_node().as_expr() {
            if let Expr::Uri(uri) = rel.uri.as_node().as_expr() {
                if let UriSegment::Variable(p) = uri.spec.first().expect("expected URI segment") {
                    let ann = p.val.annotation().expect("expected annotation");
                    let desc = ann.get_str("description").expect("expected description");
                    let req = ann.get_bool("required").expect("expected required");
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

    Ok(())
}

#[test]
fn annotate_combined() {
    let code = r#"
        # description: "some identifier"
        # required: true
        let id = num;
    "#;
    let prg = eval(code).expect("evaluation failed");

    if let Statement::Decl(decl) = prg.stmts.iter().nth(2).unwrap() {
        let ann = decl.expr.annotation().expect("expected annotation");
        let desc = ann.get_str("description").expect("expected description");
        let req = ann.get_bool("required").expect("expected required");
        assert_eq!((desc, req), ("some identifier", true));
    } else {
        panic!("expected declaration");
    }
}

#[test]
fn annotate_inline() {
    let code = r#"
        # required: true
        let id = num `title: "identifier"`;
    "#;
    let prg = eval(code).expect("evaluation failed");

    if let Statement::Decl(decl) = prg.stmts.iter().nth(1).unwrap() {
        let ann = decl.expr.annotation().expect("expected annotation");
        let title = ann.get_str("title").expect("expected title");
        let req = ann.get_bool("required").expect("expected required");
        assert_eq!((title, req), ("identifier", true));
    } else {
        panic!("expected declaration");
    }
}
