use crate::errors::{Error, Kind};
use crate::locator::Locator;
use crate::node::NodeRef;
use crate::scan::Scan;
use crate::scope::Env;
use oal_syntax::ast::{AsExpr, Program};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ModuleSet<T> {
    pub base: Locator,
    pub programs: HashMap<Locator, Program<T>>,
}

impl<T> ModuleSet<T> {
    pub fn new(base: Locator) -> Self {
        ModuleSet {
            base,
            programs: Default::default(),
        }
    }
}

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
    let mut mods = ModuleSet::new(loc.clone());
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
    let base = path.last().unwrap();
    let prg = loader(base)?;
    let mut deps = Vec::new();
    prg.scan(&mut deps, &mut Env::new(None), &mut dependency_scan)?;
    deps.into_iter().try_for_each(|dep| {
        let module = base.join(dep.as_str())?;
        if path.contains(&module) {
            Err(Error::new(Kind::CycleDetected, "loading module")
                .with(&base)
                .with(&module)
                .into())
        } else {
            let mut next = path.clone();
            next.push(module);
            recurse(mods, next, loader, compiler)
        }
    })?;
    let prog = compiler(mods, base, prg)?;
    mods.programs.insert(base.clone(), prog);
    Ok(())
}

fn dependency_scan<T, E>(acc: &mut Vec<String>, _: &mut Env<T>, node: NodeRef<T>) -> Result<(), E>
where
    T: AsExpr,
    E: From<Error>,
{
    if let NodeRef::Use(import) = node {
        acc.push(import.module.clone());
    }
    Ok(())
}
