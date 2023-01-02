/// The syntax analysis error type.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("parsing failed")]
    Parser(#[from] Box<oal_model::errors::Error>),
    #[error("value not valid for the domain: {0}")]
    Domain(String),
}

impl From<oal_model::errors::Error> for Error {
    fn from(e: oal_model::errors::Error) -> Self {
        Error::from(Box::new(e))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
