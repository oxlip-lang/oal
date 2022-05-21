use crate::errors::{Error, Kind};
use crate::locator::Locator;
use crate::scan::Scan;
use crate::scope::Env;
use oal_syntax::ast::{AsExpr, NodeRef, Program};
use std::collections::HashMap;

pub type ModuleSet<T> = HashMap<Locator, Program<T>>;

pub trait Loader<T, E>: Fn(&Locator) -> Result<Program<T>, E>
where
    T: AsExpr,
    E: From<Error>,
{
}
impl<T, E, F> Loader<T, E> for F
where
    T: AsExpr,
    E: From<Error>,
    F: Fn(&Locator) -> Result<Program<T>, E>,
{
}

pub trait Compiler<T, E>: Fn(&ModuleSet<T>, &Locator, Program<T>) -> Result<Program<T>, E>
where
    T: AsExpr,
    E: From<Error>,
{
}
impl<T, E, F> Compiler<T, E> for F
where
    T: AsExpr,
    E: From<Error>,
    F: Fn(&ModuleSet<T>, &Locator, Program<T>) -> Result<Program<T>, E>,
{
}

pub fn load<T, E, L, C>(loc: &Locator, loader: L, compiler: C) -> Result<ModuleSet<T>, E>
where
    T: AsExpr,
    E: From<Error>,
    L: Loader<T, E>,
    C: Compiler<T, E>,
{
    let mut mods = ModuleSet::new();
    recurse(&mut mods, vec![loc.clone()], &loader, &compiler)?;
    Ok(mods)
}

fn recurse<T, E, L, C>(
    mods: &mut ModuleSet<T>,
    path: Vec<Locator>,
    loader: &L,
    compiler: &C,
) -> Result<(), E>
where
    T: AsExpr,
    E: From<Error>,
    L: Loader<T, E>,
    C: Compiler<T, E>,
{
    let loc = path.last().unwrap();
    let prg = loader(loc)?;
    let mut deps = Vec::new();
    prg.scan(&mut deps, &mut Env::new(None), &mut dependency_scan)?;
    deps.into_iter().try_for_each(|dep| {
        if path.contains(&dep) {
            Err(Error::new(Kind::CycleDetected, "loading module")
                .with(&loc)
                .with(&dep)
                .into())
        } else {
            let mut next = path.clone();
            next.push(dep);
            recurse(mods, next, loader, compiler)
        }
    })?;
    let prog = compiler(mods, loc, prg)?;
    mods.insert(loc.clone(), prog);
    Ok(())
}

fn dependency_scan<T, E>(acc: &mut Vec<Locator>, _: &mut Env<T>, node: NodeRef<T>) -> Result<(), E>
where
    T: AsExpr,
{
    if let NodeRef::Use(import) = node {
        acc.push(Locator::from(&import.module))
    }
    Ok(())
}
