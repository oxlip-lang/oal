use super::{resolve::resolve, tests::mods_from};
use crate::spec::{Object, SchemaExpr, Spec, UriSegment};
use oal_syntax::atom;

fn eval(code: &str) -> anyhow::Result<Spec> {
    let mods = mods_from(code)?;
    resolve(&mods)?;
    let spec = super::eval::eval(&mods)?;
    Ok(spec)
}

#[test]
fn eval_simple() -> anyhow::Result<()> {
    let s = eval(
        r#"
        # description: "some record"
        let r = {};
        let a = /;
        res a ( put : <r> -> <r> );
    "#,
    )?;

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

#[test]
fn eval_content() -> anyhow::Result<()> {
    let s = eval(
        r#"
        let r = {};
        res / ( put : r -> <r> );
    "#,
    )?;

    assert_eq!(s.rels.len(), 1);

    Ok(())
}

#[test]
fn eval_ranges() -> anyhow::Result<()> {
    let s = eval(
        r#"
        res / ( get -> <status=200,{}> :: <status=500,media="text/plain",headers={},{}> );
    "#,
    )?;

    assert_eq!(s.rels.len(), 1);

    Ok(())
}
