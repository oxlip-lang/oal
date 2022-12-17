/// The parser error type.
#[derive(thiserror::Error, Debug)]
pub enum Error<L: crate::lexicon::Lexeme> {
    #[error("syntax analysis failed")]
    Grammar(#[from] Box<crate::grammar::ParserError<L>>),
    #[error("tokenization failed")]
    Lexicon(#[from] Box<crate::lexicon::ParserError>),
}

impl<L> From<crate::grammar::ParserError<L>> for Error<L>
where
    L: crate::lexicon::Lexeme,
{
    fn from(e: crate::grammar::ParserError<L>) -> Self {
        Error::from(Box::new(e))
    }
}

impl<L> From<crate::lexicon::ParserError> for Error<L>
where
    L: crate::lexicon::Lexeme,
{
    fn from(e: crate::lexicon::ParserError) -> Self {
        Error::from(Box::new(e))
    }
}

pub type Result<T, L> = std::result::Result<T, Error<L>>;
