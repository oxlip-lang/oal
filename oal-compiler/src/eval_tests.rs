use crate::errors;
use crate::inference::tag;
use crate::resolve::resolve;
use crate::spec::{Object, Reference, SchemaExpr, Spec, UriSegment};
use crate::tests::mods_from;
use oal_syntax::atom::{HttpStatus, Method, Operator};

fn eval(code: &str) -> anyhow::Result<Spec> {
    let mods = mods_from(code)?;
    resolve(&mods, mods.base())?;
    tag(&mods, mods.base())?;

    // Uncomment for debugging purpose:
    // println!("{:#?}", mods.main().tree().root());

    let spec = crate::eval::eval(&mods, mods.base())?;
    Ok(spec)
}

#[test]
fn eval_annotation() -> anyhow::Result<()> {
    let s = eval(
        r#"
        # description: "some record"
        let r = {};
        let a = /;
        res a ( put : <r `title: xyz`> -> <r> `description: some content`);
    "#,
    )?;

    assert_eq!(s.rels.len(), 1);
    let (i, p) = s.rels.iter().next().unwrap();

    assert_eq!(i, "/");
    assert_eq!(p.uri.path.len(), 1);
    assert_eq!(*p.uri.path.first().unwrap(), UriSegment::Literal("".into()));

    let x = p.xfers[Method::Put]
        .as_ref()
        .expect("expected transfer on HTTP PUT");

    let d = x.domain.schema.as_ref().unwrap();
    assert_eq!(d.expr, SchemaExpr::Object(Object::default()));
    assert_eq!(d.desc.as_ref().unwrap(), "some record");
    assert_eq!(d.title.as_ref().unwrap(), "xyz");

    assert_eq!(x.ranges.len(), 1);
    let c = x.ranges.values().next().unwrap();
    assert_eq!(c.desc.as_ref().unwrap(), "some content");
    let s = c.schema.as_ref().unwrap();
    assert_eq!(s.expr, SchemaExpr::Object(Object::default()));
    assert_eq!(s.desc.as_ref().unwrap(), "some record");
    assert!(s.title.is_none());

    Ok(())
}

#[test]
fn eval_composed_annotation() -> anyhow::Result<()> {
    let s = eval(
        r#"
        # description: "a number"
        # title: "a number"
        let a = num `minimum: 0`;
        res / ( get -> {
            # description: "a property"
            # required: true
            'prop a `title: "a property type"`
        } );
    "#,
    )?;

    assert_eq!(s.rels.len(), 1);
    let (_, p) = s.rels.iter().next().unwrap();
    let x = p.xfers[Method::Get]
        .as_ref()
        .expect("expected transfer on HTTP GET");
    assert_eq!(x.ranges.len(), 1);
    let c = x.ranges.values().next().unwrap();
    let s = c.schema.as_ref().unwrap();
    let SchemaExpr::Object(ref o) = s.expr else { panic!("expected an object") };
    assert_eq!(o.props.len(), 1);
    let p = o.props.first().unwrap();
    assert_eq!(p.name, "prop");
    assert_eq!(p.desc.as_ref().unwrap(), "a property");
    assert!(p.required.unwrap());
    let s = &p.schema;
    assert_eq!(s.desc.as_ref().unwrap(), "a number");
    assert_eq!(s.title.as_ref().unwrap(), "a property type");
    assert!(s.required.is_none());
    let SchemaExpr::Num(ref n) = s.expr else { panic!("expected a number") };
    assert_eq!(n.minimum.unwrap(), 0f64);

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
    let (_, p) = s.rels.iter().next().unwrap();
    let x = p.xfers[Method::Put]
        .as_ref()
        .expect("expected transfer on HTTP PUT");
    let d = x.domain.schema.as_ref().unwrap();
    assert_eq!(d.expr, SchemaExpr::Object(Object::default()));
    let r = x.ranges.values().next().unwrap().schema.as_ref().unwrap();
    assert_eq!(r.expr, SchemaExpr::Object(Object::default()));

    Ok(())
}

#[test]
fn eval_ranges() -> anyhow::Result<()> {
    let s = eval(
        r#"
        res / ( get -> <status=200, {}>
                    :: <status=500, media="text/plain", headers={ 'h str }, {}> );
    "#,
    )?;

    assert_eq!(s.rels.len(), 1);
    assert_eq!(s.rels.len(), 1);
    let (_, p) = s.rels.iter().next().unwrap();
    let x = p.xfers[Method::Get]
        .as_ref()
        .expect("expected transfer on HTTP GET");

    assert_eq!(x.ranges.len(), 2);
    let mut rs = x.ranges.iter();

    let ((s, m), c) = rs.next().unwrap();
    assert_eq!(
        *s.as_ref().expect("expected HTTP status"),
        HttpStatus::try_from(200).unwrap()
    );
    assert!(m.is_none());
    assert_eq!(
        c.schema.as_ref().unwrap().expr,
        SchemaExpr::Object(Object::default())
    );

    let ((s, m), c) = rs.next().unwrap();
    assert_eq!(
        *s.as_ref().expect("expected HTTP status"),
        HttpStatus::try_from(500).unwrap()
    );
    assert_eq!(*m.as_ref().expect("expected media type"), "text/plain");
    assert_eq!(
        c.schema.as_ref().unwrap().expr,
        SchemaExpr::Object(Object::default())
    );
    assert_eq!(c.media.as_ref().expect("expected media"), "text/plain");
    assert_eq!(
        c.status.expect("expected status"),
        HttpStatus::try_from(500).unwrap()
    );
    let o = c.headers.as_ref().expect("expected headers");
    assert_eq!(o.props.len(), 1);
    let p = &o.props[0];
    assert_eq!(p.name, "h");
    assert!(matches!(p.schema.expr, SchemaExpr::Str(_)));

    Ok(())
}

#[test]
fn eval_invalid_status() -> anyhow::Result<()> {
    let code = r#"
        res / ( get -> <status=999,{}> );
    "#;

    assert!(matches!(
        eval(code)
            .expect_err(format!("expected error evaluating: {}", code).as_str())
            .downcast_ref::<errors::Error>()
            .expect("expected compiler error")
            .kind,
        errors::Kind::Syntax(_)
    ));

    Ok(())
}

#[test]
fn eval_content_schema() -> anyhow::Result<()> {
    let s = eval(
        r#"
        let r = {
            'a num,
            'b str
        };
        res / ( get -> <r> );
    "#,
    )?;

    assert_eq!(s.rels.len(), 1);
    let (_, p) = s.rels.iter().next().unwrap();
    let x = p.xfers[Method::Get]
        .as_ref()
        .expect("expected transfer on HTTP GET");
    let r = x.ranges.values().next().unwrap().schema.as_ref().unwrap();
    let SchemaExpr::Object(o) = &r.expr else { panic!("expected an object") };
    assert_eq!(o.props.len(), 2);
    let p = &o.props[0];
    assert_eq!(p.name, "a");
    assert!(matches!(p.schema.expr, SchemaExpr::Num(_)));
    let p = &o.props[1];
    assert_eq!(p.name, "b");
    assert!(matches!(p.schema.expr, SchemaExpr::Str(_)));

    Ok(())
}

#[test]
fn eval_operation_any() -> anyhow::Result<()> {
    let s = eval(r#"res / ( get -> < { 'b [bool], 'c / } ~ num ~ uri > );"#)?;

    assert_eq!(s.rels.len(), 1);

    let (_, p) = s.rels.iter().next().unwrap();
    let x = p.xfers[Method::Get]
        .as_ref()
        .expect("expected transfer on HTTP GET");
    let r = x.ranges.values().next().unwrap().schema.as_ref().unwrap();
    let SchemaExpr::Op(op) = &r.expr else { panic!("expected an operation") };
    assert_eq!(op.op, Operator::Any);
    assert_eq!(op.schemas.len(), 3);

    let s = op.schemas.get(0).expect("expected a schema");
    let SchemaExpr::Object(o) = &s.expr else { panic!("expected an object") };
    assert_eq!(o.props.len(), 2);
    let p = &o.props[0];
    assert_eq!(p.name, "b");
    let SchemaExpr::Array(a) = &p.schema.expr else { panic!("expected an array") };
    assert!(matches!(a.item.expr, SchemaExpr::Bool(_)));
    let p = &o.props[1];
    assert_eq!(p.name, "c");
    assert!(matches!(p.schema.expr, SchemaExpr::Uri(_)));

    let s = op.schemas.get(1).expect("expected a schema");
    assert!(matches!(s.expr, SchemaExpr::Num(_)));

    let s = op.schemas.get(2).expect("expected a schema");
    assert!(matches!(s.expr, SchemaExpr::Uri(_)));

    Ok(())
}

#[test]
fn eval_operation_sum() -> anyhow::Result<()> {
    let s = eval(r#"res / ( get -> < num | str > );"#)?;

    assert_eq!(s.rels.len(), 1);

    let (_, p) = s.rels.iter().next().unwrap();
    let x = p.xfers[Method::Get]
        .as_ref()
        .expect("expected transfer on HTTP GET");
    let r = x.ranges.values().next().unwrap().schema.as_ref().unwrap();
    let SchemaExpr::Op(op) = &r.expr else { panic!("expected an operation") };
    assert_eq!(op.op, Operator::Sum);
    assert_eq!(op.schemas.len(), 2);

    let s = op.schemas.get(0).expect("expected a schema");
    assert!(matches!(s.expr, SchemaExpr::Num(_)));

    let s = op.schemas.get(1).expect("expected a schema");
    assert!(matches!(s.expr, SchemaExpr::Str(_)));

    Ok(())
}

#[test]
fn eval_uri() -> anyhow::Result<()> {
    let s = eval(r#"res /a/{ 'id num }/b?{ 'c str } ( get -> <> );"#)?;

    assert_eq!(s.rels.len(), 1);

    let (_, r) = s.rels.iter().next().unwrap();

    assert!(matches!(r.uri.path[0], UriSegment::Literal(_)));
    let UriSegment::Variable(v) = &r.uri.path[1] else { panic!("expected uri variable") };
    assert!(matches!(v.schema.expr, SchemaExpr::Num(_)));
    assert!(matches!(r.uri.path[2], UriSegment::Literal(_)));

    let o = r.uri.params.as_ref().unwrap();
    assert_eq!(o.props.len(), 1);
    let p = &o.props[0];
    assert_eq!(p.name, "c");
    assert!(matches!(p.schema.expr, SchemaExpr::Str(_)));

    Ok(())
}

#[test]
fn eval_uri_params() -> anyhow::Result<()> {
    let s = eval(r#"res / ( patch, put { 'n num } : {} -> <> );"#)?;

    assert_eq!(s.rels.len(), 1);

    let (_, r) = s.rels.iter().next().unwrap();

    assert!(r.xfers[Method::Put].is_some());
    let x = r.xfers[Method::Patch]
        .as_ref()
        .expect("expected transfer on HTTP PATCH");

    let o = x.params.as_ref().expect("expected transfer params");
    assert_eq!(o.props.len(), 1);
    let p = &o.props[0];
    assert_eq!(p.name, "n");
    assert!(matches!(p.schema.expr, SchemaExpr::Num(_)));

    Ok(())
}

#[test]
fn eval_reference() -> anyhow::Result<()> {
    let s = eval(
        r#"
        let @a = {};
        res / ( get -> @a );
    "#,
    )?;

    assert_eq!(s.rels.len(), 1);

    let r = s.rels.values().next().unwrap();

    let x = r.xfers[Method::Get]
        .as_ref()
        .expect("expected transfer on HTTP GET");

    let r = x.ranges.values().next().unwrap().schema.as_ref().unwrap();
    let SchemaExpr::Ref(i) = &r.expr else { panic!("expected an reference") };
    assert_eq!(*i, "@a");

    assert_eq!(s.refs.len(), 1);

    let Reference::Schema(r) = s.refs.values().next().unwrap();
    let SchemaExpr::Object(o) = &r.expr else { panic!("expected an object") };
    assert!(o.props.is_empty());

    Ok(())
}

#[test]
fn eval_application() -> anyhow::Result<()> {
    let s = eval(
        r#"
        let g x = x | bool;
        let f x = { 'p g int, 'q x };
        res / ( get -> < f str > );
    "#,
    )?;

    assert_eq!(s.rels.len(), 1);
    let r = s.rels.values().next().unwrap();
    let x = r.xfers[Method::Get]
        .as_ref()
        .expect("expected transfer on HTTP GET");
    let r = x.ranges.values().next().unwrap().schema.as_ref().unwrap();
    let SchemaExpr::Object(o) = &r.expr else { panic!("expected an object") };

    assert_eq!(o.props.len(), 2);

    let p = &o.props[0];
    assert_eq!(p.name, "p");
    let SchemaExpr::Op(op) = &p.schema.expr else { panic!("expected an operation") };
    assert_eq!(op.op, Operator::Sum);
    assert_eq!(op.schemas.len(), 2);

    let s = op.schemas.get(0).expect("expected a schema");
    assert!(matches!(s.expr, SchemaExpr::Int(_)));

    let s = op.schemas.get(1).expect("expected a schema");
    assert!(matches!(s.expr, SchemaExpr::Bool(_)));

    let p = &o.props[1];
    assert_eq!(p.name, "q");
    assert!(matches!(p.schema.expr, SchemaExpr::Str(_)));

    Ok(())
}

#[test]
fn eval_subexpr() -> anyhow::Result<()> {
    let s = eval(
        r#"
        let s = {} & ( {} | {} );
        res / ( get -> s );
    "#,
    )?;

    assert_eq!(s.rels.len(), 1);
    let r = s.rels.values().next().unwrap();
    let x = r.xfers[Method::Get]
        .as_ref()
        .expect("expected transfer on HTTP GET");
    let r = x.ranges.values().next().unwrap().schema.as_ref().unwrap();
    let SchemaExpr::Op(op) = &r.expr else { panic!("expected an operation") };

    assert_eq!(op.op, Operator::Join);
    assert_eq!(op.schemas.len(), 2);

    let s = op.schemas.get(0).expect("expected a schema");
    assert!(matches!(s.expr, SchemaExpr::Object(_)));

    let s = op.schemas.get(1).expect("expected a schema");
    assert!(matches!(s.expr, SchemaExpr::Op(_)));

    Ok(())
}
