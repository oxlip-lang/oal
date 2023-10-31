use oal_model::locator::Locator;
use oal_model::span::Span;
use std::fmt::{Debug, Display, Formatter};

#[derive(thiserror::Error, Debug)]
pub enum Kind {
    #[error("invalid locator: {0}")]
    Locator(#[from] oal_model::locator::Error),
    #[error("invalid YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("invalid syntax: {0}")]
    Syntax(#[from] oal_syntax::errors::Error),
    #[error("not in scope")]
    NotInScope,
    #[error("invalid type")]
    InvalidType,
    #[error("cycle detected")]
    CycleDetected,
    #[error("invalid literal")]
    InvalidLiteral,
    #[error("invalid identifier")]
    InvalidIdentifier,
    #[error("invalid module: {0}")]
    InvalidModule(Locator),
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
        self.details.push(format!("{e:?}"));
        self
    }

    pub fn at(mut self, span: Option<Span>) -> Self {
        self.span = span;
        self
    }

    pub fn span(&self) -> Option<&Span> {
        self.span.as_ref()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        if !self.msg.is_empty() {
            write!(f, ": {}", self.msg)?;
        }
        std::fmt::Result::Ok(())
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
