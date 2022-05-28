use crate::errors::{Error, Kind};
use crate::expr::TypedExpr;
use crate::inference::{constrain, substitute, tag_type, InferenceSet, TagSeq};
use crate::node::NodeRef;
use crate::reduction::reduce;
use crate::scan::Scan;
use crate::scope::Env;
use crate::transform::Transform;
use oal_syntax::ast::{AsRefNode, Expr, Operator, Primitive, Statement};
use oal_syntax::parse;

fn check_vars(
    _acc: &mut (),
    env: &mut Env<TypedExpr>,
    node: NodeRef<TypedExpr>,
) -> crate::errors::Result<()> {
    match node {
        NodeRef::Expr(e) => match e.as_node().as_expr() {
            Expr::Var(var) => match env.lookup(var) {
                None => Err(Error::new(Kind::IdentifierNotInScope, "").with(e)),
                Some(val) => match val.as_node().as_expr() {
                    Expr::Binding(_) => Ok(()),
                    _ => Err(Error::new(Kind::Unknown, "remaining free variable").with(e)),
                },
            },
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}

#[test]
fn reduce_application() {
    let code = r#"
        let b = str;
        let g x = b;
        let b = bool;
        let f x = x | num | g x;
        let a = f bool;
    "#;
    let mut prg = parse(code).expect("parsing failed");

    prg.transform(&mut TagSeq::default(), &mut Env::new(None), &mut tag_type)
        .expect("tagging failed");

    let cnt = &mut InferenceSet::new();

    prg.scan(cnt, &mut Env::new(None), &mut constrain)
        .expect("constraining failed");

    let subst = &mut cnt.unify().expect("unification failed");

    prg.transform(subst, &mut Env::new(None), &mut substitute)
        .expect("substitution failed");

    prg.transform(&mut (), &mut Env::new(None), &mut reduce)
        .expect("compilation failed");

    prg.scan(&mut (), &mut Env::new(None), &mut check_vars)
        .expect("compilation incomplete");

    match prg.stmts.iter().nth(4).unwrap() {
        Statement::Decl(d) => {
            assert_eq!(d.name.as_ref(), "a");
            match d.expr.as_node().as_expr() {
                Expr::Op(o) => {
                    assert_eq!(o.op, Operator::Sum);
                    let mut i = o.exprs.iter();
                    assert_eq!(
                        *i.next().unwrap().as_node().as_expr(),
                        Expr::Prim(Primitive::Bool)
                    );
                    assert_eq!(
                        *i.next().unwrap().as_node().as_expr(),
                        Expr::Prim(Primitive::Num)
                    );
                    assert_eq!(
                        *i.next().unwrap().as_node().as_expr(),
                        Expr::Prim(Primitive::Str)
                    );
                }
                _ => panic!("expected operation"),
            }
        }
        _ => panic!("expected declaration"),
    }
}
