use crate::Rule;

#[derive(Debug, Clone)]
pub struct ParseError {
    msg: String,
}

impl ParseError {
    fn new(msg: &str) -> ParseError {
        ParseError { msg: msg.into() }
    }
}

impl From<&str> for ParseError {
    fn from(msg: &str) -> Self {
        Self::new(msg)
    }
}

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(e: pest::error::Error<Rule>) -> Self {
        ParseError { msg: e.to_string() }
    }
}

pub type Result<T> = std::result::Result<T, ParseError>;
