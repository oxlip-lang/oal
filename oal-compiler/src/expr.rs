use crate::annotation::{Annotated, Annotation};
use crate::tag::{Tag, Tagged};
use oal_syntax::ast::{Expr, Semigroup};

#[derive(Clone, Debug, PartialEq)]
pub struct TypedExpr {
    tag: Option<Tag>,
    ann: Option<Annotation>,
    inner: Expr<TypedExpr>,
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

    fn set_annotation(&mut self, a: Annotation) {
        self.ann = Some(a);
    }
}

impl From<Expr<TypedExpr>> for TypedExpr {
    fn from(e: Expr<TypedExpr>) -> Self {
        TypedExpr {
            tag: None,
            ann: None,
            inner: e,
        }
    }
}

impl AsRef<Expr<TypedExpr>> for TypedExpr {
    fn as_ref(&self) -> &Expr<TypedExpr> {
        &self.inner
    }
}

impl AsMut<Expr<TypedExpr>> for TypedExpr {
    fn as_mut(&mut self) -> &mut Expr<TypedExpr> {
        &mut self.inner
    }
}

impl Semigroup for TypedExpr {
    /// Combines two expressions retaining the top-most annotation.
    fn combine(&mut self, with: Self) {
        self.inner = with.inner;
        self.tag = with.tag;
        if self.ann.is_none() {
            self.ann = with.ann;
        }
    }
}
