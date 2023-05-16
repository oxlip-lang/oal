use crate::errors;
use crate::inference::{check_complete, constrain, substitute, tag};
use crate::module::ModuleSet;
use crate::resolve::resolve;
use crate::tests::mods_from;
use crate::typecheck::type_check;

fn compile(code: &str) -> anyhow::Result<ModuleSet> {
    let mods = mods_from(code)?;
    resolve(&mods, mods.base())?;
    let _nvars = tag(&mods, mods.base())?;
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
        "let a = / on get -> str;",
        "let a = / on get -> <{}>;",
        "let a = /a/{ 'id num }/b?{ 'c str };",
        "let a = patch, put { 'n num } : {} -> {};",
        "let a = get { 'q str } -> {};",
        "let a = /something?{ 'q str } on get -> {};",
        "let a = 'q str; let b = /path/{a};",
        r#"let a = <status=200, media="text/plain", headers={ 'h str }, str>;"#,
        "let @a = {};",
        "res /;",
        "res / on delete -> <>;",
        "let a = ('prop str) !;",
        "let a = (<> :: <>) :: <>;",
    ];

    for c in cases {
        let mods = compile(c).expect(format!("error evaluating: {}", c).as_str());
        eprintln!("{:#?}", mods.main());
    }
}

#[test]
fn typecheck_error() {
    let cases = [
        "let a = <> ~ {};",
        "let a = / on num;",
        "let a = / on get -> ( get -> str );",
        "let a = /wrong/{ 'n [num] };",
        r#"let a = <status=num, {}>;"#,
        r#"let a = <media=str, {}>;"#,
        r#"let a = <headers=str, {}>;"#,
        "let @a = 404;",
        "let a = uri on get -> str;",
        "res num;",
        "let a = str !;",
    ];

    for c in cases {
        assert!(matches!(
            compile(c)
                .expect_err(format!("expected error evaluating: {}", c).as_str())
                .downcast_ref::<errors::Error>()
                .expect("expected compiler error")
                .kind,
            errors::Kind::InvalidType
        ));
    }
}
