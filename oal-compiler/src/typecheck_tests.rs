use crate::errors::{Error, Kind};
use crate::inference::{constrain, substitute, tag_type, InferenceSet, TagSeq};
use crate::scan::Scan;
use crate::scope::Env;
use crate::transform::Transform;
use crate::typecheck::type_check;
use crate::Program;
use oal_syntax::parse;

fn compile(code: &str) -> anyhow::Result<()> {
    let mut prg: Program = parse(code)?;

    prg.transform(&mut TagSeq::new(), &mut Env::new(), &mut tag_type)?;

    let cnt = &mut InferenceSet::new();

    prg.scan(cnt, &mut Env::new(), &mut constrain)?;

    let subst = &mut cnt.unify()?;

    prg.transform(subst, &mut Env::new(), &mut substitute)?;

    prg.scan(&mut (), &mut Env::new(), &mut type_check)?;

    anyhow::Ok(())
}

#[test]
fn typecheck_ok() -> anyhow::Result<()> {
    ["let a = { b [bool], c / } ~ num ~ uri;"]
        .iter()
        .try_for_each(|c| compile(c))
}

#[test]
fn typecheck_error() {
    assert!(["let a = <> ~ {};"].iter().map(|c| compile(c)).all(|r| r
        .unwrap_err()
        .downcast_ref::<Error>()
        .unwrap()
        .kind
        == Kind::InvalidTypes))
}
