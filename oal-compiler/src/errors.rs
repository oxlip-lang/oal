use oal_model::span::Span;
use std::fmt::{Debug, Display, Formatter};

#[derive(thiserror::Error, Debug)]
pub enum Kind {
    #[error("invalid locator")]
    Locator(#[from] oal_model::locator::Error),
    #[error("invalid YAML")]
    Yaml(#[from] serde_yaml::Error),
    #[error("invalid syntax")]
    Syntax(#[from] oal_syntax::errors::Error),
    #[error("not in scope")]
    NotInScope,
    #[error("invalid types")]
    InvalidTypes,
    #[error("cycle detected")]
    CycleDetected,
}

#[derive(Debug)]
pub struct Error {
    msg: String,
    details: Vec<String>,
    span: Option<Span>,
    pub kind: Kind,
}

impl<E: Into<Kind>> From<E> for Error {
    fn from(e: E) -> Self {
        Error {
            msg: Default::default(),
            details: Default::default(),
            span: Default::default(),
            kind: e.into(),
        }
    }
}

impl Error {
    pub fn new<S: Into<String>>(kind: Kind, msg: S) -> Self {
        Error {
            msg: msg.into(),
            details: Vec::new(),
            span: None,
            kind,
        }
    }

    pub fn with<T: Debug>(mut self, e: &T) -> Self {
        self.details.push(format!("{:?}", e));
        self
    }

    pub fn at(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{:?}: {}", self.kind, self.msg)?;
        if let Some(span) = &self.span {
            writeln!(f, "Location: {}", span)?;
        }
        if !self.details.is_empty() {
            writeln!(f, "Details:")?;
            self.details
                .iter()
                .try_for_each(|d| writeln!(f, " {}", d))?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
