use crate::errors::Result;
use oal_syntax::ast::*;

pub fn well_type(expr: &TypedExpr) -> Result<Tag> {
    match &expr.expr {
        Expr::Prim(p) => {
            let t = match p {
                Prim::Num => Tag::Number,
                Prim::Str => Tag::String,
                Prim::Bool => Tag::Boolean,
            };
            Ok(t)
        }
        Expr::Rel(rel) => {
            let uri = well_type(&rel.uri).and_then(|t| {
                if t == Tag::Uri {
                    Ok(t)
                } else {
                    Err("expected uri as relation base".into())
                }
            });
            let range = well_type(&rel.range).and_then(|t| {
                if t == Tag::Object {
                    Ok(t)
                } else {
                    Err("expected block as range".into())
                }
            });

            uri.and_then(|_| range.and_then(|_| Ok(Tag::Relation)))
        }
        Expr::Uri(uri) => {
            let r: Result<Vec<_>> = uri
                .spec
                .iter()
                .map(|s| match s {
                    UriSegment::Template(p) => well_type(&p.val).and_then(|t| {
                        if t.is_primitive() {
                            Ok(())
                        } else {
                            Err("expected prim as uri template property".into())
                        }
                    }),
                    UriSegment::Literal(_) => Ok(()),
                })
                .collect();

            r.map(|_| Tag::Uri)
        }
        Expr::Sum(sum) => {
            let sum: Result<Vec<_>> = sum.iter().map(|e| well_type(&e)).collect();

            sum.map(|sum| {
                sum.iter()
                    .reduce(|a, b| if a == b { a } else { &Tag::Any })
                    .unwrap_or(&Tag::Any)
                    .clone()
            })
        }
        Expr::Var(_) => Err("unresolved variable".into()),
        Expr::Join(join) => {
            let r: Result<Vec<_>> = join
                .iter()
                .map(|e| {
                    well_type(e).and_then(|t| {
                        if t == Tag::Object {
                            Ok(())
                        } else {
                            Err("expected block as join element".into())
                        }
                    })
                })
                .collect();

            r.map(|_| Tag::Object)
        }
        Expr::Block(_) => Ok(Tag::Object),
    }
}
