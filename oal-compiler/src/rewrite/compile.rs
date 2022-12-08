use super::eval::eval;
use super::module::ModuleSet;
use super::resolve::resolve;
use crate::errors::Result;
use crate::spec::Spec;

pub fn compile(mods: &ModuleSet) -> Result<Spec> {
    resolve(mods)?;

    // TODO: complete compilation steps

    let spec = eval(mods)?;

    Ok(spec)
}
