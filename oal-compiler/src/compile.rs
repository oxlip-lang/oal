use crate::annotation::{annotate, Annotated};
use crate::errors::Result;
use crate::inference::{constrain, substitute, tag_type, InferenceSet, TagSeq};
use crate::locator::Locator;
use crate::module::ModuleSet;
use crate::reduction::{reduce, Semigroup};
use crate::scan::Scan;
use crate::scope::Env;
use crate::tag::Tagged;
use crate::transform::Transform;
use crate::typecheck::type_check;
use oal_syntax::ast;
use oal_syntax::ast::AsExpr;

pub fn compile<T>(
    mods: &ModuleSet<T>,
    loc: &Locator,
    mut prg: ast::Program<T>,
) -> Result<ast::Program<T>>
where
    T: AsExpr + Tagged + Annotated + Semigroup,
{
    let new_env = || Env::new(Some(mods));

    prg.transform(&mut TagSeq::new(loc.clone()), &mut new_env(), &mut tag_type)?;

    let constraint = &mut InferenceSet::new();

    prg.scan(constraint, &mut new_env(), &mut constrain)?;

    let subst = &mut constraint.unify()?;

    prg.transform(subst, &mut new_env(), &mut substitute)?;

    prg.scan(&mut (), &mut new_env(), &mut type_check)?;

    prg.transform(&mut None, &mut new_env(), &mut annotate)?;

    prg.transform(&mut (), &mut new_env(), &mut reduce)?;

    Ok(prg)
}
