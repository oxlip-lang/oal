use crate::Rule;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

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

impl From<pest::error::Error<Rule>> for Error {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Error {
            msg: format!("parsing failed\n{}", e),
        }
    }
}

impl<I: Display + Hash> From<chumsky::error::Simple<I>> for Error {
    fn from(e: chumsky::error::Simple<I>) -> Self {
        Error {
            msg: format!("parsing failed\n{}", e),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
