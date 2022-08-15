use crate::annotation::{Annotated, Annotation};
use crate::reduction::Semigroup;
use crate::spec::Aliased;
use crate::tag::{Tag, Tagged};
use oal_syntax::ast::{AsMutNode, AsRefNode, Expr, NodeExpr};
use oal_syntax::atom::Ident;

#[derive(Clone, Debug, PartialEq)]
pub struct TypedExpr {
    tag: Option<Tag>,
    ann: Option<Annotation>,
    inner: NodeExpr<TypedExpr>,
    alias: Option<Ident>,
}

impl Tagged for TypedExpr {
    fn tag(&self) -> Option<&Tag> {
        self.tag.as_ref()
    }

    fn set_tag(&mut self, t: Tag) {
        self.tag = Some(t)
    }

    fn unwrap_tag(&self) -> Tag {
        self.tag.as_ref().unwrap().clone()
    }

    fn with_tag(mut self, t: Tag) -> Self {
        self.set_tag(t);
        self
    }
}

impl Annotated for TypedExpr {
    fn annotation(&self) -> Option<&Annotation> {
        self.ann.as_ref()
    }

    fn annotate(&mut self, a: Annotation) {
        self.ann.get_or_insert(Default::default()).extend(a);
    }
}

impl From<NodeExpr<TypedExpr>> for TypedExpr {
    fn from(e: NodeExpr<TypedExpr>) -> Self {
        TypedExpr {
            tag: None,
            ann: None,
            inner: e,
            alias: None,
        }
    }
}

impl AsRefNode for TypedExpr {
    fn as_node(&self) -> &NodeExpr<TypedExpr> {
        &self.inner
    }
}

impl AsMutNode for TypedExpr {
    fn as_node_mut(&mut self) -> &mut NodeExpr<TypedExpr> {
        &mut self.inner
    }
}

impl Aliased for TypedExpr {
    fn alias(&self) -> Option<&Ident> {
        self.alias.as_ref()
    }

    fn substitute(&self) -> Self {
        TypedExpr {
            alias: None,
            ..self.clone()
        }
    }
}

impl Semigroup for TypedExpr {
    /// Combines two expressions retaining annotations.
    fn combine(&mut self, with: Self) {
        // Combining a reference retains the variable identifier as alias.
        match self.as_node().as_expr() {
            Expr::Var(var) if var.is_reference() => self.alias = Some(var.clone()),
            _ => {}
        }
        // Once set the alias is immutable by combination.
        if self.alias.is_none() {
            self.alias = with.alias;
        }
        self.inner = with.inner;
        self.tag = with.tag;
        if let Some(ann) = &mut self.ann {
            if let Some(other) = with.ann {
                ann.extend(other);
            }
        } else {
            self.ann = with.ann;
        }
    }
}
