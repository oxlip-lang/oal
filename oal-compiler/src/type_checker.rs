use crate::errors::Result;
use oal_syntax::ast::*;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TypeTag {
    Number,
    String,
    Boolean,
    Relation,
    Object,
    Uri,
    Any,
}

impl TypeTag {
    fn is_primitive(&self) -> bool {
        *self == Self::Number || *self == Self::String || *self == Self::Boolean
    }
}

pub fn well_type(expr: &TypeExpr) -> Result<TypeTag> {
    match expr {
        TypeExpr::Prim(p) => {
            let t = match p {
                TypePrim::Num => TypeTag::Number,
                TypePrim::Str => TypeTag::String,
                TypePrim::Bool => TypeTag::Boolean,
            };
            Ok(t)
        }
        TypeExpr::Rel(rel) => {
            let uri = well_type(&rel.uri).and_then(|t| {
                if t == TypeTag::Uri {
                    Ok(t)
                } else {
                    Err("expected uri as relation base".into())
                }
            });
            let range = well_type(&rel.range).and_then(|t| {
                if t == TypeTag::Object {
                    Ok(t)
                } else {
                    Err("expected block as range".into())
                }
            });

            uri.and_then(|_| range.and_then(|_| Ok(TypeTag::Relation)))
        }
        TypeExpr::Uri(uri) => {
            let r: Result<Vec<_>> = uri
                .spec
                .iter()
                .map(|s| match s {
                    UriSegment::Template(p) => well_type(&p.expr).and_then(|t| {
                        if t.is_primitive() {
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
                    .reduce(|a, b| if a == b { a } else { &TypeTag::Any })
                    .unwrap_or(&TypeTag::Any)
                    .clone()
            })
        }
        TypeExpr::Var(_) => Err("unresolved variable".into()),
        TypeExpr::Join(join) => {
            let r: Result<Vec<_>> = join
                .iter()
                .map(|e| {
                    well_type(e).and_then(|t| {
                        if t == TypeTag::Object {
                            Ok(())
                        } else {
                            Err("expected block as join element".into())
                        }
                    })
                })
                .collect();

            r.map(|_| TypeTag::Object)
        }
        TypeExpr::Block(_) => Ok(TypeTag::Object),
    }
}
