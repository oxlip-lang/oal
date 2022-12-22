use super::eval::eval;
use super::infer::{constrain, substitute, tag, check_tags};
use super::module::ModuleSet;
use super::resolve::resolve;
use crate::errors::Result;
use crate::spec::Spec;
use crate::Locator;

pub fn compile(mods: &ModuleSet, loc: &Locator) -> Result<Spec> {
    // Resolve variable and function references.
    resolve(mods, loc)?;
    // Tag expressions with concrete and variable types.
    tag(mods, loc)?;
    // Collect the set of type inference equations.
    let eqs = constrain(mods, loc)?;
    // Unify the inference set.
    let set = eqs.unify()?;
    // Substitute tags in each class of equivalence with the representative tag.
    substitute(mods, loc, &set)?;
    // Check for remaining tag variables.
    check_tags(mods, loc)?;
    // TODO: type check
    let spec = eval(mods, loc)?;
    Ok(spec)
}
