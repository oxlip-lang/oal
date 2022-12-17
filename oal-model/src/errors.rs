/// The parser error type.
#[derive(thiserror::Error, Debug)]
pub enum Error<L: crate::lexicon::Lexeme> {
    #[error("syntax analysis failed")]
    Parsing(#[from] Box<crate::grammar::ParserError<L>>),
    #[error("tokenization failed")]
    Lexing(#[from] Box<crate::lexicon::ParserError>),
}

pub type Result<T, L> = std::result::Result<T, Error<L>>;
