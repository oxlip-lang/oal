use crate::errors::Result;
use crate::locator::Locator;
use crate::module::load;
use crate::{compile, Program};
use oal_syntax::ast::{AsRefNode, Expr, Operator, Statement};
use oal_syntax::parse;
use std::path::Path;

#[test]
fn compile_module() {
    let module = &Locator::from(Path::new("module.oal"));
    let main = &Locator::from(Path::new("main.oal"));
    let loader = |m: &Locator| -> Result<Program> {
        if m == module {
            Ok(parse("let f x = x | num;").expect("parsing failed"))
        } else if m == main {
            Ok(parse(r#"use "module.oal"; let id = f str;"#).expect("parsing failed"))
        } else {
            unreachable!()
        }
    };
    let mods = load(main, loader, compile).expect("loading failed");

    assert_eq!(mods.len(), 2);

    let p = mods.get(main).expect("expected main program");
    if let Statement::Decl(d) = p.stmts.last().expect("expected statement") {
        if let Expr::Op(op) = d.expr.as_node().as_expr() {
            assert_eq!(op.op, Operator::Sum);
        } else {
            panic!("expected operation")
        }
    } else {
        panic!("expected declaration")
    }
}
