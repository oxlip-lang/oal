use crate::scope::Env;
use oal_syntax::ast::{Expr, Ident, Prim, Tag, TypedExpr};

#[test]
fn environment_scopes() {
    let mut e = Env::new();
    let id = Ident::from("a");
    let bool_expr = TypedExpr {
        tag: Some(Tag::Primitive),
        inner: Expr::Prim(Prim::Bool),
    };
    let num_expr = TypedExpr {
        tag: Some(Tag::Primitive),
        inner: Expr::Prim(Prim::Num),
    };

    assert!(!e.exists(&id));

    e.declare(&id, &bool_expr);

    assert_eq!(e.head().len(), 1);
    assert!(e.exists(&id));
    assert_eq!(*e.lookup(&id).expect("lookup failed"), bool_expr);

    e.open();

    assert!(e.head().is_empty());
    assert!(!e.exists(&id));
    assert_eq!(*e.lookup(&id).expect("lookup failed"), bool_expr);

    e.declare(&id, &num_expr);

    assert_eq!(e.head().len(), 1);
    assert!(e.exists(&id));
    assert_eq!(*e.lookup(&id).expect("lookup failed"), num_expr);

    e.close();

    assert!(e.exists(&id));
    assert_eq!(*e.lookup(&id).expect("lookup failed"), bool_expr);
}
