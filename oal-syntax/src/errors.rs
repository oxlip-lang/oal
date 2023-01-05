/// The syntax analysis error type.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("syntax analysis failed")]
    Grammar(#[from] Box<oal_model::grammar::ParserError<crate::lexer::Token>>),
    #[error("tokenization failed")]
    Lexicon(#[from] Box<oal_model::lexicon::ParserError>),
    #[error("value not valid for the domain: {0}")]
    Domain(String),
}

pub type Result<T> = std::result::Result<T, Error>;
