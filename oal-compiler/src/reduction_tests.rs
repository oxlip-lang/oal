use crate::errors::Error;
use crate::inference::{constrain, substitute, tag_type, TagSeq, TypeConstraint};
use crate::reduce;
use crate::scan::Scan;
use crate::scope::Env;
use crate::transform::Transform;
use oal_syntax::ast::{Expr, Operator, Primitive, Statement, TypedExpr};
use oal_syntax::parse;

fn check_vars(acc: &mut (), env: &mut Env, e: &TypedExpr) -> crate::errors::Result<()> {
    e.as_ref().scan(acc, env, check_vars)?;
    match e.as_ref() {
        Expr::Var(var) => match env.lookup(var) {
            None => Err(Error::new("identifier not in scope").with_expr(e.as_ref())),
            Some(val) => match val.as_ref() {
                Expr::Binding(_) => Ok(()),
                _ => Err(Error::new("remaining free variable").with_expr(e.as_ref())),
            },
        },
        _ => Ok(()),
    }
}

#[test]
fn compile_application() {
    let code = r#"
        let b = str;
        let g x = b;
        let b = bool;
        let f x = x | num | g x;
        let a = f bool;
    "#;
    let mut prg = parse(code.into()).expect("parsing failed");

    prg.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)
        .expect("tagging failed");

    let cnt = &mut TypeConstraint::new();

    prg.scan(cnt, &mut Env::new(), constrain)
        .expect("constraining failed");

    let subst = &mut cnt.unify().expect("unification failed");

    prg.transform(subst, &mut Env::new(), substitute)
        .expect("substitution failed");

    prg.transform(&mut (), &mut Env::new(), reduce)
        .expect("compilation failed");

    prg.scan(&mut (), &mut Env::new(), check_vars)
        .expect("compilation incomplete");

    match prg.stmts.iter().nth(4).unwrap() {
        Statement::Decl(d) => {
            assert_eq!(d.name.as_ref(), "a");
            match d.expr.as_ref() {
                Expr::Op(o) => {
                    assert_eq!(o.op, Operator::Sum);
                    let mut i = o.exprs.iter();
                    assert_eq!(i.next().unwrap().inner, Expr::Prim(Primitive::Bool));
                    assert_eq!(i.next().unwrap().inner, Expr::Prim(Primitive::Num));
                    assert_eq!(i.next().unwrap().inner, Expr::Prim(Primitive::Str));
                }
                _ => panic!("expected operation"),
            }
        }
        _ => panic!("expected declaration"),
    }
}
