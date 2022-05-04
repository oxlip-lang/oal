use crate::inference::{constrain, substitute, tag_type, InferenceSet, TagSeq};
use crate::scan::Scan;
use crate::scope::Env;
use crate::transform::Transform;
use crate::typecheck::type_check;
use crate::Program;
use oal_syntax::parse;

fn compile(code: &str) -> anyhow::Result<()> {
    let mut prg: Program = parse(code.into())?;

    prg.transform(&mut TagSeq::new(), &mut Env::new(), &mut tag_type)?;

    let cnt = &mut InferenceSet::new();

    prg.scan(cnt, &mut Env::new(), &mut constrain)?;

    let subst = &mut cnt.unify()?;

    prg.transform(subst, &mut Env::new(), &mut substitute)?;

    prg.scan(&mut (), &mut Env::new(), &mut type_check)?;

    anyhow::Ok(())
}

#[test]
fn typecheck_simple() -> anyhow::Result<()> {
    compile("let a = { b [bool], c / } ~ num ~ uri;")
}
