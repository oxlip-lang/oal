use super::eval::eval;
use super::infer::tag;
use super::module::ModuleSet;
use super::resolve::resolve;
use crate::errors::Result;
use crate::spec::Spec;
use crate::Locator;

pub fn compile(mods: &ModuleSet, loc: &Locator) -> Result<Spec> {
    resolve(mods, loc)?;
    tag(mods, loc)?;
    // TODO: infer types
    let spec = eval(mods, loc)?;
    Ok(spec)
}
