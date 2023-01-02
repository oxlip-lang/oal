use crate::errors::Result;
use crate::inference::{check_complete, constrain, substitute, tag};
use crate::module::ModuleSet;
use crate::resolve::resolve;
use crate::typecheck::type_check;
use oal_model::locator::Locator;

pub fn compile(mods: &ModuleSet, loc: &Locator) -> Result<()> {
    // Resolve variable and function references.
    resolve(mods, loc)?;
    // Tag expressions with concrete and variable types.
    let _nvars = tag(mods, loc)?;
    // Collect the set of type inference equations.
    let eqs = constrain(mods, loc)?;
    // Unify the inference set.
    let set = eqs.unify()?;
    // Substitute tags in each class of equivalence with the representative tag.
    substitute(mods, loc, &set)?;
    // Check for remaining type tag variables.
    check_complete(mods, loc)?;
    // Check type tags against expectations.
    type_check(mods, loc)?;
    Ok(())
}
