use crate::scope::Env;
use oal_syntax::ast::{Expr, Ident, Primitive, Tag, Tagged, TypedExpr};

#[test]
fn environment_scopes() {
    let mut e = Env::new();
    let id = Ident::from("a");
    let bool_expr = TypedExpr::from(Expr::Prim(Primitive::Bool)).with_tag(Tag::Primitive);
    let num_expr = TypedExpr::from(Expr::Prim(Primitive::Num)).with_tag(Tag::Primitive);

    assert!(!e.exists(&id));

    e.declare(&id, &bool_expr);

    assert_eq!(e.head().len(), 1);
    assert!(e.exists(&id));
    assert_eq!(*e.lookup(&id).expect("lookup failed"), bool_expr);

    e.within(|e| {
        assert!(e.head().is_empty());
        assert!(!e.exists(&id));
        assert_eq!(*e.lookup(&id).expect("lookup failed"), bool_expr);

        e.declare(&id, &num_expr);

        assert_eq!(e.head().len(), 1);
        assert!(e.exists(&id));
        assert_eq!(*e.lookup(&id).expect("lookup failed"), num_expr);
    });

    assert!(e.exists(&id));
    assert_eq!(*e.lookup(&id).expect("lookup failed"), bool_expr);
}
