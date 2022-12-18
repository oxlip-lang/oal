use crate::expr::TypedExpr;
use crate::scope::Env;
use crate::inference::tag::{Tag, Tagged};
use oal_syntax::ast::Expr;
use oal_syntax::atom::{Ident, Primitive};

#[test]
fn environment_scopes() {
    let mut e = Env::new(None);
    let id = Ident::from("a");
    let bool_expr =
        TypedExpr::from(Expr::Prim(Primitive::Boolean).into_node()).with_tag(Tag::Primitive);
    let num_expr =
        TypedExpr::from(Expr::Prim(Primitive::Number).into_node()).with_tag(Tag::Primitive);

    assert!(!e.exists(&id));

    e.declare(id.clone(), bool_expr.clone());

    assert_eq!(e.head().len(), 1);
    assert!(e.exists(&id));
    assert_eq!(*e.lookup(&id).expect("lookup failed"), bool_expr);

    e.within(|e| {
        assert!(e.head().is_empty());
        assert!(!e.exists(&id));
        assert_eq!(*e.lookup(&id).expect("lookup failed"), bool_expr);

        e.declare(id.clone(), num_expr.clone());

        assert_eq!(e.head().len(), 1);
        assert!(e.exists(&id));
        assert_eq!(*e.lookup(&id).expect("lookup failed"), num_expr);
    });

    assert!(e.exists(&id));
    assert_eq!(*e.lookup(&id).expect("lookup failed"), bool_expr);
}
