use crate::errors::Result;
use oal_syntax::ast::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TypeTag {
    Prim,
    Uri,
    Rel,
    Block,
    Unknown,
}

pub fn well_type(expr: &TypeExpr) -> Result<TypeTag> {
    match expr {
        TypeExpr::Prim(_) => Ok(TypeTag::Prim),
        TypeExpr::Rel(rel) => {
            let uri = well_type(&rel.uri).and_then(|t| {
                if let TypeTag::Uri = t {
                    Ok(t)
                } else {
                    Err("expected uri as relation base".into())
                }
            });
            let range = well_type(&rel.range).and_then(|t| {
                if let TypeTag::Block = t {
                    Ok(t)
                } else {
                    Err("expected block as range".into())
                }
            });

            uri.and_then(|_| range.and_then(|_| Ok(TypeTag::Rel)))
        }
        TypeExpr::Uri(uri) => {
            let r: Result<Vec<_>> = uri
                .spec
                .iter()
                .map(|s| match s {
                    UriSegment::Template(p) => well_type(&p.expr).and_then(|t| {
                        if let TypeTag::Prim = t {
                            Ok(())
                        } else {
                            Err("expected prim as uri template property".into())
                        }
                    }),
                    UriSegment::Literal(_) => Ok(()),
                })
                .collect();

            r.map(|_| TypeTag::Uri)
        }
        TypeExpr::Sum(sum) => {
            let sum: Result<Vec<_>> = sum.iter().map(|e| well_type(e)).collect();

            sum.map(|sum| {
                sum.iter()
                    .reduce(|a, b| if a == b { a } else { &TypeTag::Unknown })
                    .unwrap_or(&TypeTag::Unknown)
                    .clone()
            })
        }
        TypeExpr::Var(_) => Err("unresolved variable".into()),
        TypeExpr::Join(join) => {
            let r: Result<Vec<_>> = join
                .iter()
                .map(|e| {
                    well_type(e).and_then(|t| {
                        if t == TypeTag::Block {
                            Ok(())
                        } else {
                            Err("expected block as join element".into())
                        }
                    })
                })
                .collect();

            r.map(|_| TypeTag::Block)
        }
        TypeExpr::Block(_) => Ok(TypeTag::Block),
    }
}
