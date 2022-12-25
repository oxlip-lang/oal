use super::module::ModuleSet;
use super::tree::{get_tag, Core};
use crate::errors::{Error, Kind, Result};
use crate::inference::tag::Tag;
use crate::locator::Locator;
use oal_syntax::atom;
use oal_syntax::rewrite::lexer as lex;
use oal_syntax::rewrite::parser as syn;

fn check_operation(op: syn::VariadicOp<Core>) -> Result<()> {
    match op.operator() {
        atom::Operator::Join => {
            if !op.operands().all(|o| get_tag(o) == Tag::Object) {
                return Err(Error::new(Kind::InvalidTypes, "ill-formed join").with(&op));
            }
        }
        atom::Operator::Any | atom::Operator::Sum => {
            if !op.operands().all(|o| get_tag(o).is_schema()) {
                return Err(Error::new(Kind::InvalidTypes, "ill-formed alternative").with(&op));
            }
        }
        atom::Operator::Range => {
            if !op.operands().all(|o| get_tag(o).is_schema_like()) {
                return Err(Error::new(Kind::InvalidTypes, "ill-formed ranges").with(&op));
            }
        }
    }
    Ok(())
}

fn check_content(content: syn::Content<Core>) -> Result<()> {
    for meta in content.meta() {
        match meta.tag() {
            lex::Content::Media => {
                if get_tag(meta.rhs()) != Tag::Text {
                    return Err(Error::new(Kind::InvalidTypes, "ill-formed media").with(&meta));
                }
            }
            lex::Content::Headers => {
                if !get_tag(meta.rhs()).is_schema() {
                    return Err(Error::new(Kind::InvalidTypes, "ill-formed headers").with(&meta));
                }
            }
            lex::Content::Status => {
                if !get_tag(meta.rhs()).is_status_like() {
                    return Err(Error::new(Kind::InvalidTypes, "ill-formed status").with(&meta));
                }
            }
        }
    }
    if let Some(body) = content.body() {
        if !get_tag(body).is_schema() {
            return Err(Error::new(Kind::InvalidTypes, "ill-formed body").with(&content));
        }
    }
    Ok(())
}

fn check_transfer(xfer: syn::Transfer<Core>) -> Result<()> {
    if let Some(domain) = xfer.domain() {
        if !get_tag(domain.inner()).is_schema_like() {
            return Err(Error::new(Kind::InvalidTypes, "ill-formed domain").with(&domain));
        }
    }
    if !get_tag(xfer.range()).is_schema_like() {
        return Err(Error::new(Kind::InvalidTypes, "ill-formed range").with(&xfer.range()));
    }
    Ok(())
}

fn check_relation(relation: syn::Relation<Core>) -> Result<()> {
    if get_tag(relation.uri().inner()) != Tag::Uri {
        return Err(Error::new(Kind::InvalidTypes, "ill-formed uri").with(&relation.uri()));
    }
    if !relation.transfers().all(|t| get_tag(t) == Tag::Transfer) {
        return Err(Error::new(Kind::InvalidTypes, "ill-formed transfers").with(&relation));
    }
    Ok(())
}

fn check_uri(uri: syn::UriTemplate<Core>) -> Result<()> {
    if !uri.segments().all(|s| match s {
        syn::UriSegment::Element(_) => true,
        syn::UriSegment::Variable(v) => {
            matches!(get_tag(v.inner()), Tag::Property(t) if *t == Tag::Primitive)
        }
    }) {
        return Err(Error::new(Kind::InvalidTypes, "ill-formed uri").with(&uri));
    }
    Ok(())
}

fn check_array(array: syn::Array<Core>) -> Result<()> {
    if !get_tag(array.inner()).is_schema() {
        return Err(Error::new(Kind::InvalidTypes, "ill-formed array").with(&array));
    }
    Ok(())
}

fn check_property(prop: syn::Property<Core>) -> Result<()> {
    if !get_tag(prop.rhs()).is_schema() {
        return Err(Error::new(Kind::InvalidTypes, "ill-formed property").with(&prop));
    }
    Ok(())
}

fn check_object(object: syn::Object<Core>) -> Result<()> {
    if !object
        .properties()
        .all(|p| matches!(get_tag(p), Tag::Property(_)))
    {
        return Err(Error::new(Kind::InvalidTypes, "ill-formed object").with(&object));
    }
    Ok(())
}

fn check_declaration(decl: syn::Declaration<Core>) -> Result<()> {
    if decl.ident().is_reference() && !get_tag(decl.rhs()).is_schema() {
        return Err(Error::new(Kind::InvalidTypes, "ill-formed reference").with(&decl));
    }
    Ok(())
}

fn check_resource(res: syn::Resource<Core>) -> Result<()> {
    if !get_tag(res.relation()).is_relation_like() {
        return Err(Error::new(Kind::InvalidTypes, "ill-formed resource").with(&res));
    }
    Ok(())
}

/// Returns an error if there is at least one type mismatch.
pub fn type_check(mods: &ModuleSet, loc: &Locator) -> Result<()> {
    let module = mods.get(loc).expect("module not found");

    for node in module.tree().root().descendants() {
        if let Some(operation) = syn::VariadicOp::cast(node) {
            check_operation(operation)?;
        } else if let Some(content) = syn::Content::cast(node) {
            check_content(content)?;
        } else if let Some(xfer) = syn::Transfer::cast(node) {
            check_transfer(xfer)?;
        } else if let Some(relation) = syn::Relation::cast(node) {
            check_relation(relation)?;
        } else if let Some(uri) = syn::UriTemplate::cast(node) {
            check_uri(uri)?;
        } else if let Some(array) = syn::Array::cast(node) {
            check_array(array)?;
        } else if let Some(prop) = syn::Property::cast(node) {
            check_property(prop)?;
        } else if let Some(object) = syn::Object::cast(node) {
            check_object(object)?;
        } else if let Some(decl) = syn::Declaration::cast(node) {
            check_declaration(decl)?;
        } else if let Some(res) = syn::Resource::cast(node) {
            check_resource(res)?;
        }
    }

    Ok(())
}
