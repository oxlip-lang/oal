use crate::errors::Result;
use crate::inference::{constrain, substitute, tag};
use crate::module::ModuleSet;
use crate::resolve::resolve;
use crate::typecheck::{cycles_check, type_check};
use oal_model::locator::Locator;

/// Runs all compilation phases.
pub fn compile(mods: &ModuleSet, loc: &Locator) -> Result<()> {
    // Resolve variable and function references. Returns the graph of definitions.
    let graph = resolve(mods, loc)?;
    // Tag expressions with concrete and variable types.
    let _nvars = tag(mods, loc)?;
    // Collect the set of type inference equations.
    let eqs = constrain(mods, loc)?;
    // Unify the inference set.
    let set = eqs.unify()?;
    // Substitute tags in each class of equivalence with the representative tag.
    substitute(mods, loc, &set)?;
    // Validates points of recursion in the graph of definitions.
    cycles_check(graph, mods)?;
    // Check type tags against expectations.
    type_check(mods, loc)?;
    Ok(())
}
