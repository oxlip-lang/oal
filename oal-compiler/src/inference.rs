pub mod disjoin;
pub mod tag;
pub mod unify;

use crate::errors::{Error, Kind, Result};
use crate::node::{NodeMut, NodeRef};
use crate::scope::Env;
use oal_syntax::ast::{AsExpr, Expr, Literal, Operator, UriSegment};
use tag::{FuncTag, Tag, Tagged};

fn from_lit(l: &Literal) -> Tag {
    match l {
        Literal::Text(_) => Tag::Text,
        Literal::Number(_) => Tag::Number,
        Literal::Status(_) => Tag::Status,
    }
}

pub fn tag_type<T>(seq: &mut tag::Seq, env: &mut Env<T>, node_ref: NodeMut<T>) -> Result<()>
where
    T: AsExpr + Tagged,
{
    if let NodeMut::Expr(expr) = node_ref {
        let node = expr.as_node();
        let span = node.span;
        match node.as_expr() {
            Expr::Lit(l) => {
                expr.set_tag(from_lit(l));
                Ok(())
            }
            Expr::Prim(_) => {
                expr.set_tag(Tag::Primitive);
                Ok(())
            }
            Expr::Rel(_) => {
                expr.set_tag(Tag::Relation);
                Ok(())
            }
            Expr::Uri(_) => {
                expr.set_tag(Tag::Uri);
                Ok(())
            }
            Expr::Property(_) => {
                expr.set_tag(Tag::Property);
                Ok(())
            }
            Expr::Object(_) => {
                expr.set_tag(Tag::Object);
                Ok(())
            }
            Expr::Content(_) => {
                expr.set_tag(Tag::Content);
                Ok(())
            }
            Expr::Xfer(_) => {
                expr.set_tag(Tag::Transfer);
                Ok(())
            }
            Expr::Array(_) => {
                expr.set_tag(Tag::Array);
                Ok(())
            }
            Expr::Op(operation) => {
                let tag = match operation.op {
                    Operator::Join => Tag::Object,
                    Operator::Any => Tag::Any,
                    Operator::Sum => Tag::Var(seq.next()),
                    Operator::Range => Tag::Content,
                };
                expr.set_tag(tag);
                Ok(())
            }
            Expr::Var(var) => match env.lookup(var) {
                None => Err(Error::new(Kind::NotInScope, "tag").with(expr)),
                Some(val) => {
                    expr.set_tag(val.unwrap_tag());
                    Ok(())
                }
            },
            Expr::Lambda(_) | Expr::Binding(_) => {
                expr.set_tag(Tag::Var(seq.next()));
                Ok(())
            }
            Expr::App(application) => match env.lookup(&application.name) {
                None => Err(Error::new(Kind::NotInScope, "tag").with(expr)),
                Some(val) => {
                    if let Expr::Lambda(l) = val.as_node().as_expr() {
                        expr.set_tag(l.body.unwrap_tag());
                        Ok(())
                    } else {
                        Err(Error::new(Kind::NotAFunction, "").with(expr))
                    }
                }
            },
        }
        .map_err(|err| err.at(span))
    } else {
        Ok(())
    }
}

pub fn substitute<T: Tagged>(
    subst: &mut disjoin::Set,
    _env: &mut Env<T>,
    node: NodeMut<T>,
) -> Result<()> {
    if let NodeMut::Expr(e) = node {
        e.set_tag(subst.substitute(e.tag().unwrap()))
    }
    Ok(())
}

pub fn constrain<T>(
    c: &mut unify::InferenceSet,
    env: &mut Env<T>,
    node_ref: NodeRef<T>,
) -> Result<()>
where
    T: AsExpr + Tagged,
{
    if let NodeRef::Expr(expr) = node_ref {
        let node = expr.as_node();
        let span = node.span;
        match node.as_expr() {
            Expr::Lit(lit) => {
                c.push(expr.unwrap_tag(), from_lit(lit), span);
                Ok(())
            }
            Expr::Prim(_) => {
                c.push(expr.unwrap_tag(), Tag::Primitive, span);
                Ok(())
            }
            Expr::Rel(rel) => {
                c.push(rel.uri.unwrap_tag(), Tag::Uri, rel.uri.as_node().span);
                for xfer in rel.xfers.iter() {
                    c.push(xfer.unwrap_tag(), Tag::Transfer, xfer.as_node().span);
                }
                c.push(expr.unwrap_tag(), Tag::Relation, span);
                Ok(())
            }
            Expr::Uri(uri) => {
                for seg in uri.path.iter() {
                    if let UriSegment::Variable(var) = seg {
                        c.push(var.unwrap_tag(), Tag::Property, var.as_node().span);
                    }
                }
                if let Some(params) = &uri.params {
                    c.push(params.unwrap_tag(), Tag::Object, params.as_node().span);
                }
                c.push(expr.unwrap_tag(), Tag::Uri, span);
                Ok(())
            }
            Expr::Property(_) => {
                c.push(expr.unwrap_tag(), Tag::Property, span);
                Ok(())
            }
            Expr::Object(obj) => {
                for prop in obj.props.iter() {
                    c.push(prop.unwrap_tag(), Tag::Property, prop.as_node().span);
                }
                c.push(expr.unwrap_tag(), Tag::Object, span);
                Ok(())
            }
            Expr::Content(cnt) => {
                cnt.headers
                    .iter()
                    .for_each(|h| c.push(h.unwrap_tag(), Tag::Object, h.as_node().span));
                cnt.media
                    .iter()
                    .for_each(|m| c.push(m.unwrap_tag(), Tag::Text, m.as_node().span));
                c.push(expr.unwrap_tag(), Tag::Content, span);
                Ok(())
            }
            Expr::Xfer(xfer) => {
                if let Some(params) = &xfer.params {
                    c.push(params.unwrap_tag(), Tag::Object, params.as_node().span);
                }
                c.push(expr.unwrap_tag(), Tag::Transfer, span);
                Ok(())
            }
            Expr::Array(_) => {
                c.push(expr.unwrap_tag(), Tag::Array, span);
                Ok(())
            }
            Expr::Op(operation) => {
                let operator = operation.op;
                for op in operation.into_iter() {
                    match operator {
                        Operator::Join => c.push(op.unwrap_tag(), Tag::Object, op.as_node().span),
                        Operator::Sum => {
                            c.push(expr.unwrap_tag(), op.unwrap_tag(), op.as_node().span)
                        }
                        Operator::Any | Operator::Range => {}
                    }
                }
                match operator {
                    Operator::Join => c.push(expr.unwrap_tag(), Tag::Object, span),
                    Operator::Any => c.push(expr.unwrap_tag(), Tag::Any, span),
                    Operator::Range => c.push(expr.unwrap_tag(), Tag::Content, span),
                    Operator::Sum => {}
                }
                Ok(())
            }
            Expr::Lambda(lambda) => {
                let bindings = lambda.bindings.iter().map(|b| b.unwrap_tag()).collect();
                let range = lambda.body.unwrap_tag().into();
                c.push(
                    expr.unwrap_tag(),
                    Tag::Func(FuncTag { bindings, range }),
                    span,
                );
                Ok(())
            }
            Expr::App(application) => match env.lookup(&application.name) {
                None => Err(Error::new(Kind::NotInScope, "constraint").with(expr)),
                Some(val) => {
                    let bindings = application.args.iter().map(|a| a.unwrap_tag()).collect();
                    let range = expr.unwrap_tag().into();
                    c.push(
                        val.unwrap_tag(),
                        Tag::Func(FuncTag { bindings, range }),
                        val.as_node().span,
                    );
                    Ok(())
                }
            },
            Expr::Var(_) => Ok(()),
            Expr::Binding(_) => Ok(()),
        }
        .map_err(|err| err.at(span))
    } else {
        Ok(())
    }
}
