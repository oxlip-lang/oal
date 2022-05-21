use crate::compile::compile;
use crate::spec::{Expr, Object, Spec, Uri, UriSegment};
use crate::{Locator, ModuleSet, Program};
use oal_syntax::parse;

fn eval(code: &str) -> anyhow::Result<Spec> {
    let loc = Locator::try_from("test:main")?;
    let mods = ModuleSet::new(loc.clone());
    let prg: Program = parse(code)?;
    let prg = compile(&mods, &loc, prg)?;
    let spec = Spec::try_from(&prg)?;

    anyhow::Ok(spec)
}

#[test]
fn uri_pattern() {
    let uri = Uri {
        spec: vec![UriSegment::Literal("".into())],
    };

    assert_eq!(uri.pattern(), "/");
}

#[test]
fn evaluate_simple() -> anyhow::Result<()> {
    let code = r#"
        # description: "some record"
        let r = {};
        res / ( put : <r> -> <r> );
    "#;

    let s = eval(code)?;

    assert_eq!(s.rels.len(), 1);

    let (i, p) = s.rels.iter().next().unwrap();

    assert_eq!(i, "/");
    assert_eq!(p.uri.spec.len(), 1);
    assert_eq!(*p.uri.spec.first().unwrap(), UriSegment::Literal("".into()));

    if let Some(x) = &p.xfers[oal_syntax::ast::Method::Put] {
        let d = x.domain.schema.as_ref().unwrap();
        assert_eq!(d.expr, Expr::Object(Object::default()));
        assert_eq!(d.desc, Some("some record".to_owned()));
        let r = x.range.schema.as_ref().unwrap();
        assert_eq!(r.expr, Expr::Object(Object::default()));
        assert_eq!(r.desc, Some("some record".to_owned()));
    } else {
        panic!("expected transfer on HTTP PUT");
    }

    anyhow::Ok(())
}

#[test]
fn evaluate_content() -> anyhow::Result<()> {
    let code = r#"
        let r = {};
        res / ( put : r -> <r> );
    "#;

    let s = eval(code)?;

    assert_eq!(s.rels.len(), 1);

    anyhow::Ok(())
}
