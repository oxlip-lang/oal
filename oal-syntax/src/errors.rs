type GrammarError = oal_model::grammar::ParserError;
type LexiconError = oal_model::lexicon::ParserError;

/// The syntax analysis error type.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("syntax analysis failed: {0}")]
    Grammar(#[from] GrammarError),
    #[error("tokenization failed: {0}")]
    Lexicon(#[from] LexiconError),
    #[error("value not valid for the domain")]
    Domain,
}

pub type Result<T> = std::result::Result<T, Error>;
