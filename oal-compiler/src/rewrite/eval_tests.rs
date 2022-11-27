use super::{eval::eval, resolve::resolve, tests::mods_from};
use crate::spec::{Object, SchemaExpr, UriSegment};
use oal_syntax::atom;

#[test]
#[ignore]
fn eval_simple() -> anyhow::Result<()> {
    let mods = mods_from(
        r#"
    # description: "some record"
    let r = {};
    let a = /;
    res a ( put : <r> -> <r> );
    "#,
    )?;

    resolve(&mods)?;

    let s = eval(&mods)?;

    println!("{:#?}", s);

    assert_eq!(s.rels.len(), 1);

    let (i, p) = s.rels.iter().next().unwrap();

    assert_eq!(i, "/");
    assert_eq!(p.uri.path.len(), 1);
    assert_eq!(*p.uri.path.first().unwrap(), UriSegment::Literal("".into()));

    if let Some(x) = &p.xfers[atom::Method::Put] {
        let d = x.domain.schema.as_ref().unwrap();
        assert_eq!(d.expr, SchemaExpr::Object(Object::default()));
        assert_eq!(d.desc, Some("some record".to_owned()));
        let r = x.ranges.values().next().unwrap().schema.as_ref().unwrap();
        assert_eq!(r.expr, SchemaExpr::Object(Object::default()));
        assert_eq!(r.desc, Some("some record".to_owned()));
    } else {
        panic!("expected transfer on HTTP PUT");
    }

    Ok(())
}
