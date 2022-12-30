use crate::rewrite::lexer::Token;

/// The syntax analysis error type.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("parsing failed")]
    Parser(#[from] Box<oal_model::errors::Error<Token>>),
    #[error("value not valid for the domain: {0}")]
    Domain(String),
}

impl From<oal_model::errors::Error<Token>> for Error {
    fn from(e: oal_model::errors::Error<Token>) -> Self {
        Error::from(Box::new(e))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
