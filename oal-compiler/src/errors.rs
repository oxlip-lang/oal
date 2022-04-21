use crate::inference::TypeEquation;
use oal_syntax::ast::{Expr, Tag};

#[derive(Debug, Clone, Default)]
pub struct Error {
    msg: String,
    expr: Option<Expr>,
    tag: Option<Tag>,
    eq: Option<TypeEquation>,
}

impl Error {
    pub fn new(msg: &str) -> Error {
        Error {
            msg: msg.into(),
            ..Default::default()
        }
    }

    pub fn with_expr(mut self, e: &Expr) -> Self {
        self.expr = Some(e.clone());
        self
    }

    pub fn with_tag(mut self, t: Option<&Tag>) -> Self {
        self.tag = t.cloned();
        self
    }

    pub fn with_eq(mut self, eq: &TypeEquation) -> Self {
        self.eq = Some(eq.clone());
        self
    }
}

pub type Result<T> = std::result::Result<T, Error>;
