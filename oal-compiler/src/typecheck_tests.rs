use crate::errors::{Error, Kind};
use crate::inference::{constrain, substitute, tag_type, InferenceSet, TagSeq};
use crate::scan::Scan;
use crate::scope::Env;
use crate::transform::Transform;
use crate::typecheck::type_check;
use crate::Program;
use oal_syntax::parse;

fn eval(code: &str) -> anyhow::Result<()> {
    let mut prg: Program = parse(code)?;

    prg.transform(&mut TagSeq::default(), &mut Env::new(None), &mut tag_type)?;

    let cnt = &mut InferenceSet::new();

    prg.scan(cnt, &mut Env::new(None), &mut constrain)?;

    let subst = &mut cnt.unify()?;

    prg.transform(subst, &mut Env::new(None), &mut substitute)?;

    prg.scan(&mut (), &mut Env::new(None), &mut type_check)?;

    anyhow::Ok(())
}

#[test]
fn typecheck_ok() {
    let cases = [
        "let a = { b [bool], c / } ~ num ~ uri;",
        "let a = / ( get -> str );",
        "let a = / ( get -> <{}> );",
    ];

    for c in cases {
        eval(c).expect(format!("error evaluating: {}", c).as_str());
    }
}

#[test]
fn typecheck_error() {
    let cases = [
        "let a = <> ~ {};",
        "let a = / ( num );",
        "let a = / ( get -> ( get -> str ) );",
    ];

    for c in cases {
        assert_eq!(
            eval(c)
                .expect_err(format!("expected error evaluating: {}", c).as_str())
                .downcast_ref::<Error>()
                .unwrap()
                .kind,
            Kind::InvalidTypes
        );
    }
}
