use crate::definition::Internal;
use crate::eval::{AnnRef, Expr};
use crate::spec::{Uri, UriSegment};
use crate::stdlib;

#[test]
fn concat() {
    let c = stdlib::Concat {};
    let left = Uri {
        path: vec![UriSegment::Literal("a".into())],
        params: None,
        example: None,
    };
    let right = Uri {
        path: vec![UriSegment::Literal("b".into())],
        params: None,
        example: None,
    };
    let args = vec![
        (Expr::Uri(left.into()), AnnRef::default()),
        (Expr::Uri(right.into()), AnnRef::default()),
    ];
    let (expr, _) = c.eval(args, AnnRef::default()).expect("evaluation failed");
    let Expr::Uri(uri) = expr else { panic!("expected a uri") };
    assert_eq!(uri.pattern(), "/a/b");
}
