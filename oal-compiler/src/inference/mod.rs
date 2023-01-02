pub mod disjoin;
pub mod tag;
pub mod unify;

#[cfg(test)]
mod tests;

use crate::errors::{Error, Kind, Result};
use crate::module::ModuleSet;
use crate::tree::{definition, get_tag, set_tag};
use oal_model::locator::Locator;
use oal_syntax::atom;
use oal_syntax::lexer as lex;
use oal_syntax::parser as syn;
use tag::{FuncTag, Seq, Tag};
use unify::InferenceSet;

fn literal_tag(t: &lex::TokenValue) -> Tag {
    match t {
        lex::TokenValue::HttpStatus(_) => Tag::Status,
        lex::TokenValue::Number(_) => Tag::Number,
        lex::TokenValue::Symbol(_) => Tag::Text,
        _ => panic!("unexpected token for literal {:?}", t),
    }
}

/// Assigns type tags to all expressions in the given module.
/// Returns the number of tag variables allocated.
pub fn tag(mods: &ModuleSet, loc: &Locator) -> Result<usize> {
    let module = mods.get(loc).expect("module not found");
    let mut seq = Seq::new(loc.clone());

    for node in module.tree().root().descendants() {
        if syn::Literal::cast(node).is_some() {
            set_tag(node, literal_tag(node.token().value()));
        } else if syn::Primitive::cast(node).is_some() {
            set_tag(node, Tag::Primitive);
        } else if syn::Relation::cast(node).is_some() {
            set_tag(node, Tag::Relation);
        } else if syn::UriTemplate::cast(node).is_some() {
            set_tag(node, Tag::Uri);
        } else if syn::Object::cast(node).is_some() {
            set_tag(node, Tag::Object);
        } else if syn::Content::cast(node).is_some() {
            set_tag(node, Tag::Content);
        } else if syn::Transfer::cast(node).is_some() {
            set_tag(node, Tag::Transfer);
        } else if syn::Array::cast(node).is_some() {
            set_tag(node, Tag::Array);
        } else if let Some(op) = syn::VariadicOp::cast(node) {
            let operator = op.operator();
            let tag = match operator {
                atom::Operator::Join => Tag::Object,
                atom::Operator::Any => Tag::Any,
                atom::Operator::Sum => Tag::Var(seq.next()),
                atom::Operator::Range => Tag::Content,
            };
            set_tag(node, tag);
        } else if syn::Application::cast(node).is_some()
            || syn::Variable::cast(node).is_some()
            || syn::Binding::cast(node).is_some()
            || syn::Terminal::cast(node).is_some()
            || syn::SubExpression::cast(node).is_some()
            || syn::Declaration::cast(node).is_some()
            || syn::Property::cast(node).is_some()
        {
            set_tag(node, Tag::Var(seq.next()));
        }
    }

    Ok(seq.len())
}

/// Returns the set of type inference equations for the given module.
pub fn constrain(mods: &ModuleSet, loc: &Locator) -> Result<InferenceSet> {
    let module = mods.get(loc).expect("module not found");
    let mut set = InferenceSet::new();

    for node in module.tree().root().descendants() {
        if let Some(rel) = syn::Relation::cast(node) {
            set.push(get_tag(rel.uri().node()), Tag::Uri, rel.uri().node().span());
            for xfer in rel.transfers() {
                set.push(get_tag(xfer), Tag::Transfer, xfer.span());
            }
        } else if let Some(uri) = syn::UriTemplate::cast(node) {
            for seg in uri.segments() {
                if let syn::UriSegment::Variable(var) = seg {
                    let tag = Tag::Property(Box::new(Tag::Primitive));
                    set.push(get_tag(var.inner()), tag, var.inner().span())
                }
            }
            if let Some(params) = uri.params() {
                set.push(get_tag(params.node()), Tag::Object, params.node().span());
            }
        } else if let Some(prop) = syn::Property::cast(node) {
            let rhs = get_tag(prop.rhs()).into();
            set.push(get_tag(node), Tag::Property(rhs), node.span());
        } else if let Some(cnt) = syn::Content::cast(node) {
            for meta in cnt.meta() {
                if let Some(t) = match meta.tag() {
                    lex::Content::Headers => Some(Tag::Object),
                    lex::Content::Media => Some(Tag::Text),
                    lex::Content::Status => None,
                } {
                    set.push(get_tag(meta.rhs()), t, meta.rhs().span());
                }
            }
        } else if let Some(xfer) = syn::Transfer::cast(node) {
            if let Some(params) = xfer.params() {
                set.push(get_tag(params.node()), Tag::Object, params.node().span());
            }
        } else if let Some(op) = syn::VariadicOp::cast(node) {
            for operand in op.operands() {
                if let Some(t) = match op.operator() {
                    atom::Operator::Range | atom::Operator::Any => None,
                    atom::Operator::Join => Some(Tag::Object),
                    atom::Operator::Sum => Some(get_tag(node)),
                } {
                    set.push(get_tag(operand), t, operand.span());
                }
            }
        } else if let Some(decl) = syn::Declaration::cast(node) {
            let bindings: Vec<_> = decl.bindings().map(|b| get_tag(b.node())).collect();
            let tag = if bindings.is_empty() {
                get_tag(decl.rhs())
            } else {
                Tag::Func(FuncTag {
                    bindings,
                    range: get_tag(decl.rhs()).into(),
                })
            };
            set.push(get_tag(node), tag, node.span());
        } else if let Some(app) = syn::Application::cast(node) {
            let definition = definition(mods, node).ok_or_else(|| {
                Error::new(Kind::NotInScope, "function is not defined").with(&node)
            })?;
            let bindings = app.arguments().map(|a| get_tag(a.node())).collect();
            let range = get_tag(node).into();
            set.push(
                get_tag(definition),
                Tag::Func(FuncTag { bindings, range }),
                node.span(),
            );
        } else if syn::Variable::cast(node).is_some() {
            let definition = definition(mods, node).ok_or_else(|| {
                Error::new(Kind::NotInScope, "variable is not defined").with(&node)
            })?;
            set.push(get_tag(node), get_tag(definition), node.span());
        } else if let Some(term) = syn::Terminal::cast(node) {
            set.push(get_tag(node), get_tag(term.inner()), node.span());
        } else if let Some(expr) = syn::SubExpression::cast(node) {
            set.push(get_tag(node), get_tag(expr.inner()), node.span());
        }
    }

    Ok(set)
}

/// Substitutes tags in each class of equivalence with the representative tag.
pub fn substitute(mods: &ModuleSet, loc: &Locator, set: &disjoin::Set) -> Result<()> {
    let module = mods.get(loc).expect("module not found");

    for node in module.tree().root().descendants() {
        let mut core = node.syntax().core_mut();
        if let Some(tag) = core.tag().map(|t| set.substitute(t)) {
            core.set_tag(tag);
        }
    }

    Ok(())
}

fn has_variable(tag: &Tag) -> bool {
    match tag {
        Tag::Var(_) => true,
        Tag::Property(t) => has_variable(t),
        Tag::Func(f) => has_variable(f.range.as_ref()) || f.bindings.iter().any(has_variable),
        _ => false,
    }
}

/// Returns an error if there is at least one remaining tag variable.
pub fn check_complete(mods: &ModuleSet, loc: &Locator) -> Result<()> {
    let module = mods.get(loc).expect("module not found");

    for node in module.tree().root().descendants() {
        if let Some(tag) = node.syntax().core_ref().tag() {
            if has_variable(tag) {
                return Err(Error::new(Kind::InvalidTypes, "unsolved tag variable").with(&node));
            }
        }
    }

    Ok(())
}
