use oal_model::span::Span;
use std::fmt::{Debug, Display, Formatter};

#[derive(thiserror::Error, Debug, Default)]
pub enum Kind {
    #[error("YAML")]
    Yaml(#[from] serde_yaml::Error),
    #[error("syntax")]
    Syntax(#[from] oal_syntax::errors::Error),
    #[error("model")]
    Model(#[from] oal_model::errors::Error),
    #[error("not in scope")]
    NotInScope,
    #[error("not a function")]
    NotAFunction,
    #[error("invalid types")]
    InvalidTypes,
    #[error("conflict")]
    Conflict,
    #[error("unexpected expression")]
    UnexpectedExpression,
    #[error("cycle detected")]
    CycleDetected,
    #[error("invalid HTTP status")]
    InvalidHttpStatus,
    #[default]
    #[error("unknown error")]
    Unknown,
}

#[derive(Debug, Default)]
pub struct Error {
    msg: String,
    details: Vec<String>,
    span: Option<Span>,
    pub kind: Kind,
}

impl<E: Into<Kind>> From<E> for Error {
    fn from(e: E) -> Self {
        Error {
            kind: e.into(),
            ..Default::default()
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
