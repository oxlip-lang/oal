use crate::errors::{Error, Kind, Result};
use crate::inference::tag::Tag;
use crate::module::ModuleSet;
use crate::tree::{Core, NRef};
use oal_model::grammar::AbstractSyntaxNode;
use oal_model::locator::Locator;
use oal_syntax::atom;
use oal_syntax::parser as syn;

struct TagWrap(Tag, bool);

impl TagWrap {
    fn is_recursive(&self) -> bool {
        self.1
    }

    fn is_variable(&self) -> bool {
        matches!(self.0, Tag::Var(_))
    }

    fn is_schema(&self) -> bool {
        matches!(
            self.0,
            Tag::Primitive
                | Tag::Relation
                | Tag::Object
                | Tag::Array
                | Tag::Uri
                | Tag::Any
                | Tag::Var(_)
        )
    }

    fn is_referential(&self) -> bool {
        matches!(self.0, |Tag::Relation| Tag::Object
            | Tag::Array
            | Tag::Any
            | Tag::Var(_))
    }

    fn is_content_like(&self) -> bool {
        self.is_schema() || self.0 == Tag::Content
    }

    fn is_status_like(&self) -> bool {
        matches!(self.0, Tag::Status | Tag::Number | Tag::Var(_))
    }

    fn is_relation_like(&self) -> bool {
        matches!(self.0, Tag::Relation | Tag::Uri | Tag::Var(_))
    }

    fn is_object(&self) -> bool {
        matches!(self.0, Tag::Object | Tag::Var(_))
    }

    fn is_property(&self) -> bool {
        matches!(self.0, Tag::Property(_) | Tag::Var(_))
    }

    fn is_primitive_property(&self) -> bool {
        self.is_variable() || matches!(&self.0, Tag::Property(t) if *t.as_ref() == Tag::Primitive)
    }

    fn is_text(&self) -> bool {
        matches!(self.0, Tag::Text | Tag::Var(_))
    }

    fn is_uri(&self) -> bool {
        matches!(self.0, Tag::Uri | Tag::Var(_))
    }

    fn is_transfer(&self) -> bool {
        matches!(self.0, Tag::Transfer | Tag::Var(_))
    }
}

fn get_tag(n: NRef) -> TagWrap {
    TagWrap(crate::tree::get_tag(n), n.syntax().core_ref().is_recursive)
}

fn check_variadic_operation(op: syn::VariadicOp<Core>) -> Result<()> {
    match op.operator() {
        atom::VariadicOperator::Join => {
            if !op.operands().all(|o| get_tag(o).is_object()) {
                return Err(Error::new(Kind::InvalidType, "ill-formed join").with(&op));
            }
        }
        atom::VariadicOperator::Any | atom::VariadicOperator::Sum => {
            if !op.operands().all(|o| get_tag(o).is_schema()) {
                return Err(Error::new(Kind::InvalidType, "ill-formed alternative").with(&op));
            }
        }
        atom::VariadicOperator::Range => {
            if !op.operands().all(|o| get_tag(o).is_content_like()) {
                return Err(Error::new(Kind::InvalidType, "ill-formed ranges").with(&op));
            }
        }
    }
    Ok(())
}

fn check_unary_operation(op: syn::UnaryOp<Core>) -> Result<()> {
    match op.operator() {
        atom::UnaryOperator::Optional | atom::UnaryOperator::Required => {
            if !get_tag(op.operand()).is_property() {
                return Err(Error::new(Kind::InvalidType, "ill-formed optionality").with(&op));
            }
        }
    }
    Ok(())
}

fn check_content(content: syn::Content<Core>) -> Result<()> {
    for meta in content.meta().into_iter().flatten() {
        match meta.kind() {
            syn::ContentTagKind::Media => {
                if !get_tag(meta.rhs()).is_text() {
                    return Err(Error::new(Kind::InvalidType, "ill-formed media").with(&meta));
                }
            }
            syn::ContentTagKind::Headers => {
                if !get_tag(meta.rhs()).is_schema() {
                    return Err(Error::new(Kind::InvalidType, "ill-formed headers").with(&meta));
                }
            }
            syn::ContentTagKind::Status => {
                if !get_tag(meta.rhs()).is_status_like() {
                    return Err(Error::new(Kind::InvalidType, "ill-formed status").with(&meta));
                }
            }
        }
    }
    if let Some(body) = content.body() {
        if !get_tag(body).is_schema() {
            return Err(Error::new(Kind::InvalidType, "ill-formed body").with(&content));
        }
    }
    Ok(())
}

fn check_transfer(xfer: syn::Transfer<Core>) -> Result<()> {
    if let Some(domain) = xfer.domain() {
        if !get_tag(domain.inner()).is_content_like() {
            return Err(Error::new(Kind::InvalidType, "ill-formed domain").with(&domain));
        }
    }
    if !get_tag(xfer.range()).is_content_like() {
        return Err(Error::new(Kind::InvalidType, "ill-formed range").with(&xfer.range()));
    }
    Ok(())
}

fn check_relation(relation: syn::Relation<Core>) -> Result<()> {
    if !get_tag(relation.uri().inner()).is_uri() {
        return Err(Error::new(Kind::InvalidType, "ill-formed uri").with(&relation.uri()));
    }
    if !relation.transfers().all(|t| get_tag(t).is_transfer()) {
        return Err(Error::new(Kind::InvalidType, "ill-formed transfers").with(&relation));
    }
    Ok(())
}

fn check_uri(uri: syn::UriTemplate<Core>) -> Result<()> {
    if !uri.segments().all(|s| match s {
        syn::UriSegment::Element(_) => true,
        syn::UriSegment::Variable(v) => get_tag(v.inner()).is_primitive_property(),
    }) {
        return Err(Error::new(Kind::InvalidType, "ill-formed uri").with(&uri));
    }
    Ok(())
}

fn check_array(array: syn::Array<Core>) -> Result<()> {
    if !get_tag(array.inner()).is_schema() {
        return Err(Error::new(Kind::InvalidType, "ill-formed array").with(&array));
    }
    Ok(())
}

fn check_property(prop: syn::Property<Core>) -> Result<()> {
    if !get_tag(prop.rhs()).is_schema() {
        return Err(Error::new(Kind::InvalidType, "ill-formed property").with(&prop));
    }
    Ok(())
}

fn check_object(object: syn::Object<Core>) -> Result<()> {
    if !object.properties().all(|p| get_tag(p).is_property()) {
        return Err(Error::new(Kind::InvalidType, "ill-formed object").with(&object));
    }
    Ok(())
}

fn check_declaration(decl: syn::Declaration<Core>) -> Result<()> {
    let rhs = get_tag(decl.rhs());
    if decl.ident().is_reference() && !rhs.is_schema() {
        return Err(
            Error::new(Kind::InvalidType, "ill-formed reference, not a schema").with(&decl),
        );
    }
    if get_tag(decl.node()).is_recursive() {
        if !rhs.is_schema() {
            return Err(
                Error::new(Kind::InvalidType, "ill-formed recursion, not a schema").with(&decl),
            );
        }
        if !rhs.is_referential() {
            return Err(
                Error::new(Kind::InvalidType, "ill-formed recursion, not referential").with(&decl),
            );
        }
        if decl.has_bindings() {
            return Err(Error::new(Kind::InvalidType, "ill-formed lambda, recursive").with(&decl));
        }
    }
    Ok(())
}

fn check_resource(res: syn::Resource<Core>) -> Result<()> {
    if !get_tag(res.relation()).is_relation_like() {
        return Err(Error::new(Kind::InvalidType, "ill-formed resource").with(&res));
    }
    Ok(())
}

fn check_recursion(rec: syn::Recursion<Core>) -> Result<()> {
    let tag = get_tag(rec.node());
    if !tag.is_schema() {
        return Err(Error::new(Kind::InvalidType, "ill-formed recursion, not a schema").with(&rec));
    }
    if !tag.is_referential() {
        return Err(
            Error::new(Kind::InvalidType, "ill-formed recursion, not referential").with(&rec),
        );
    }
    Ok(())
}

/// Returns an error if there is at least one type mismatch.
pub fn type_check(mods: &ModuleSet, loc: &Locator) -> Result<()> {
    let module = mods.get(loc).expect("module not found");

    for node in module.root().descendants() {
        if let Some(operation) = syn::VariadicOp::cast(node) {
            check_variadic_operation(operation)
        } else if let Some(operation) = syn::UnaryOp::cast(node) {
            check_unary_operation(operation)
        } else if let Some(content) = syn::Content::cast(node) {
            check_content(content)
        } else if let Some(xfer) = syn::Transfer::cast(node) {
            check_transfer(xfer)
        } else if let Some(relation) = syn::Relation::cast(node) {
            check_relation(relation)
        } else if let Some(uri) = syn::UriTemplate::cast(node) {
            check_uri(uri)
        } else if let Some(array) = syn::Array::cast(node) {
            check_array(array)
        } else if let Some(prop) = syn::Property::cast(node) {
            check_property(prop)
        } else if let Some(object) = syn::Object::cast(node) {
            check_object(object)
        } else if let Some(decl) = syn::Declaration::cast(node) {
            check_declaration(decl)
        } else if let Some(res) = syn::Resource::cast(node) {
            check_resource(res)
        } else if let Some(rec) = syn::Recursion::cast(node) {
            check_recursion(rec)
        } else {
            Ok(())
        }
        .map_err(|err| err.at(node.span()))?;
    }

    Ok(())
}
