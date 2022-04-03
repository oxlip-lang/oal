use crate::compile;
use crate::errors::Result;
use crate::inference::{constrain, substitute, tag_type, TagSeq, TypeConstraint};
use crate::scan::Scan;
use crate::scope::Env;
use crate::transform::Transform;
use oal_syntax::ast::{Doc, Expr, Rel, Res, Stmt, TypedExpr};

pub fn eval(mut doc: Doc) -> Result<Vec<Rel>> {
    doc.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)?;

    let constraint = &mut TypeConstraint::new();

    doc.scan(constraint, &mut Env::new(), constrain)?;

    let subst = &mut constraint.unify()?;

    doc.transform(subst, &mut Env::new(), substitute)?;

    doc.transform(&mut (), &mut Env::new(), compile)?;

    doc.stmts
        .into_iter()
        .filter_map(|s| match s {
            Stmt::Res(Res {
                rel:
                    TypedExpr {
                        inner: Expr::Rel(r),
                        tag: _,
                    },
            }) => Some(Ok(r)),
            _ => None,
        })
        .collect()
}
