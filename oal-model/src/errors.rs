/// The parser error type.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("syntax analysis failed")]
    Grammar(Box<dyn std::error::Error + Send + Sync>),
    #[error("tokenization failed")]
    Lexicon(Box<crate::lexicon::ParserError>),
    #[error("invalid URL")]
    Url(#[from] url::ParseError),
    #[error("invalid path")]
    Path,
    #[error("input/output error")]
    IO(#[from] std::io::Error),
}

impl<L> From<crate::grammar::ParserError<L>> for Error
where
    L: crate::lexicon::Lexeme,
{
    fn from(e: crate::grammar::ParserError<L>) -> Self {
        Error::Grammar(Box::new(e))
    }
}

impl From<crate::lexicon::ParserError> for Error {
    fn from(e: crate::lexicon::ParserError) -> Self {
        Error::Lexicon(Box::new(e))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
