use crate::Rule;

#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
}

impl Error {
    pub fn new(msg: &str) -> Error {
        Error { msg: msg.into() }
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self::new(msg)
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Error { msg: e.to_string() }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
