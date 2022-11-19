use crate::errors::Result;
use crate::rewrite::module::ModuleSet;
use crate::rewrite::resolve::resolve;
use crate::spec::Spec;

#[allow(unused_variables, unreachable_code)]
pub fn compile(mods: &ModuleSet) -> Result<Spec> {
    resolve(mods)?;

    // TODO: implement compilation steps
    let spec = todo!();

    Ok(spec)
}
