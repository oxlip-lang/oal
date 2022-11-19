use super::module::ModuleSet;
use super::resolve::resolve;
use crate::errors::Result;
use crate::spec::Spec;

#[allow(unused_variables, unreachable_code)]
pub fn compile(mods: &ModuleSet) -> Result<Spec> {
    resolve(mods)?;

    // TODO: implement compilation steps
    let spec = todo!();

    Ok(spec)
}
