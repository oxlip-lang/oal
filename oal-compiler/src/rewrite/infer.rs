use super::module::ModuleSet;
use super::tree::definition;
use crate::errors::Result;
use crate::inference::tag::{Seq, Tag};
use crate::locator::Locator;
use oal_syntax::rewrite::lexer as lex;
use oal_syntax::rewrite::parser as syn;

fn literal_tag(t: &lex::TokenValue) -> Tag {
    match t {
        lex::TokenValue::HttpStatus(_) => Tag::Status,
        lex::TokenValue::Number(_) => Tag::Number,
        lex::TokenValue::Symbol(_) => Tag::Text,
        _ => panic!("unexpected token for literal {:?}", t),
    }
}

pub fn tag(mods: &ModuleSet, loc: &Locator) -> Result<()> {
    let module = mods.get(loc).expect("module not found");
    let mut seq = Seq::new(loc.clone());

    for node in module.tree().root().descendants() {
        let tag = if syn::Literal::cast(node).is_some() {
            Some(literal_tag(node.token().value()))
        } else if syn::Primitive::cast(node).is_some() {
            Some(Tag::Primitive)
        } else if syn::Relation::cast(node).is_some() {
            Some(Tag::Relation)
        } else if syn::UriTemplate::cast(node).is_some() {
            Some(Tag::Uri)
        } else if syn::Property::cast(node).is_some() {
            Some(Tag::Property)
        } else if syn::Object::cast(node).is_some() {
            Some(Tag::Object)
        } else if syn::Content::cast(node).is_some() {
            Some(Tag::Content)
        } else if syn::Transfer::cast(node).is_some() {
            Some(Tag::Transfer)
        } else if syn::Array::cast(node).is_some() {
            Some(Tag::Array)
        } else if let Some(op) = syn::VariadicOp::cast(node) {
            Some(match op.operator() {
                lex::Operator::Ampersand => Tag::Object,
                lex::Operator::Tilde => Tag::Any,
                lex::Operator::VerticalBar => Tag::Var(seq.next()),
                lex::Operator::DoubleColon => Tag::Content,
                _ => panic!("unexpected operator {:?}", op.operator()),
            })
        } else if syn::Declaration::cast(node).is_some() {
            Some(Tag::Var(seq.next()))
        } else if syn::Binding::cast(node).is_some() {
            Some(Tag::Var(seq.next()))
        } else if syn::Variable::cast(node).is_some() {
            let definition = definition(mods, node).expect("variable is not defined");
            Some(definition.syntax().core_ref().unwrap_tag())
        } else if syn::Application::cast(node).is_some() {
            let definition = definition(mods, node).expect("function is not defined");
            Some(definition.syntax().core_ref().unwrap_tag())
        } else {
            None
        };
        if let Some(t) = tag {
            node.syntax().core_mut().set_tag(t);
        }
    }
    Ok(())
}
