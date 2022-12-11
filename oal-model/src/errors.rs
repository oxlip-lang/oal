use std::fmt::{Display, Formatter};
use std::hash::Hash;

// TODO: maybe use 'thiserror' crate.
#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
}

impl Error {
    pub fn new(msg: &str) -> Error {
        Error { msg: msg.into() }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Error {}

impl<I, S> From<chumsky::error::Simple<I, S>> for Error
where
    I: Display + Hash,
    S: chumsky::Span,
{
    fn from(e: chumsky::error::Simple<I, S>) -> Self {
        Error {
            msg: format!("parsing failed: {}", e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
