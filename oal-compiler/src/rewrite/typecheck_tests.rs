use super::infer::{check_complete, constrain, substitute, tag};
use super::module::ModuleSet;
use super::resolve::resolve;
use super::tests::mods_from;
use super::typecheck::type_check;
use crate::errors;

fn compile(code: &str) -> anyhow::Result<ModuleSet> {
    let mods = mods_from(code)?;
    resolve(&mods, mods.base())?;
    tag(&mods, mods.base())?;
    let eqs = constrain(&mods, mods.base())?;
    let set = eqs.unify()?;
    substitute(&mods, mods.base(), &set)?;
    check_complete(&mods, mods.base())?;
    type_check(&mods, mods.base())?;
    Ok(mods)
}

#[test]
fn typecheck_ok() {
    let cases = [
        "let a = { 'b [bool], 'c / } ~ num ~ uri;",
        "let a = / ( get -> str );",
        "let a = / ( get -> <{}> );",
        "let a = /a/{ 'id num }/b?{ 'c str };",
        "let a = patch, put { 'n num } : {} -> {};",
        "let a = get { 'q str } -> {};",
        "let a = /something?{ 'q str } ( get -> {} );",
        "let a = 'q str; let b = /path/{a};",
        r#"let a = <status=200, media="text/plain", headers={ 'h str }, str>;"#,
        "let @a = {};",
        "res /;",
        "res / ( delete -> <> );",
    ];

    for c in cases {
        compile(c).expect(format!("error evaluating: {}", c).as_str());
    }
}

#[test]
fn typecheck_error() {
    let cases = [
        "let a = <> ~ {};",
        "let a = / ( num );",
        "let a = / ( get -> ( get -> str ) );",
        "let a = /wrong/{ 'n [num] };",
        r#"let a = <status=num, {}>;"#,
        r#"let a = <media=str, {}>;"#,
        r#"let a = <headers=str, {}>;"#,
        "let @a = 404;",
        "let a = uri ( get -> str );",
        "res num;",
    ];

    for c in cases {
        assert!(matches!(
            compile(c)
                .expect_err(format!("expected error evaluating: {}", c).as_str())
                .downcast_ref::<errors::Error>()
                .expect("expected compiler error")
                .kind,
            errors::Kind::InvalidTypes
        ));
    }
}
