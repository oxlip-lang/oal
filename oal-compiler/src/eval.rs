use crate::errors::{Error, Result};
use crate::inference::{constrain, substitute, tag_type, TagSeq, TypeConstraint};
use crate::reduce;
use crate::scan::Scan;
use crate::scope::Env;
use crate::transform::Transform;
use oal_syntax::ast::{Block, Doc, Expr, Method, Res, Stmt, TypedExpr, Uri};
use oal_syntax::try_each::TryEach;
use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct PathOperation {
    pub domain: Option<Block>,
    pub range: Block,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathItem {
    pub uri: Uri,
    pub ops: HashMap<Method, PathOperation>,
}

pub type PathPattern = String;
pub type Paths = HashMap<PathPattern, PathItem>;

#[derive(Clone, Debug, PartialEq)]
pub struct Spec {
    pub paths: Paths,
}

pub fn evaluate(mut doc: Doc) -> Result<Spec> {
    doc.transform(&mut TagSeq::new(), &mut Env::new(), tag_type)?;

    let constraint = &mut TypeConstraint::new();

    doc.scan(constraint, &mut Env::new(), constrain)?;

    let subst = &mut constraint.unify()?;

    doc.transform(subst, &mut Env::new(), substitute)?;

    doc.transform(&mut (), &mut Env::new(), reduce)?;

    let mut paths: Paths = HashMap::new();

    doc.stmts
        .try_each(|stmt| match stmt {
            Stmt::Res(Res {
                rel:
                    TypedExpr {
                        inner: Expr::Rel(rel),
                        tag: _,
                    },
            }) => {
                let uri = match rel.uri.inner {
                    Expr::Uri(uri) => uri,
                    _ => panic!("expected uri expression"),
                };

                let item = paths.entry(uri.pattern()).or_insert(PathItem {
                    uri,
                    ops: Default::default(),
                });

                let domain = rel.domain.map(|d| match d.inner {
                    Expr::Block(block) => block,
                    _ => panic!("expected record expression"),
                });

                let range = match rel.range.inner {
                    Expr::Block(block) => block,
                    _ => panic!("expected record expression"),
                };

                rel.methods.try_each(|method| match item.ops.entry(method) {
                    Vacant(v) => {
                        v.insert(PathOperation {
                            domain: domain.clone(),
                            range: range.clone(),
                        });
                        Ok(())
                    }
                    _ => Err(Error::new("duplicated path operation")),
                })
            }
            _ => Ok(()),
        })
        .map(|_| Spec { paths })
}
