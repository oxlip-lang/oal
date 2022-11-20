use super::{eval::eval, resolve::resolve, tests::mods_from};

#[test]
fn eval_simple() -> anyhow::Result<()> {
    let mods = mods_from(
        r#"
    let a = /;
    res a (get -> <>);
    "#,
    )?;

    resolve(&mods)?;
    eval(&mods)?;

    Ok(())
}
