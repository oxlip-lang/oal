type GrammarError = Box<oal_model::grammar::ParserError<crate::lexer::Token>>;
type LexiconError = Box<oal_model::lexicon::ParserError>;

/// The syntax analysis error type.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("syntax analysis failed\nLocation: {}", .0.span())]
    Grammar(#[from] GrammarError),
    #[error("tokenization failed\nLocation: {}", .0.span())]
    Lexicon(#[from] LexiconError),
    #[error("value not valid for the domain: {0}")]
    Domain(String),
}

pub type Result<T> = std::result::Result<T, Error>;
