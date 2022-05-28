use crate::expr::TypedExpr;
use crate::scope::Env;
use crate::tag::{Tag, Tagged};
use oal_syntax::ast::{Expr, Primitive};
use oal_syntax::terminal::Ident;

#[test]
fn environment_scopes() {
    let mut e = Env::new(None);
    let id = Ident::from("a");
    let bool_expr =
        TypedExpr::from(Expr::Prim(Primitive::Bool).into_node()).with_tag(Tag::Primitive);
    let num_expr = TypedExpr::from(Expr::Prim(Primitive::Num).into_node()).with_tag(Tag::Primitive);

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
