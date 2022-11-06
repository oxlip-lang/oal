use crate::Rule;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Error {
    msg: String,
    source: Option<Box<dyn std::error::Error + Sync + Send>>,
}

impl Error {
    pub fn new<S: Into<String>>(msg: S) -> Self {
        Error {
            msg: msg.into(),
            source: None,
        }
    }

    pub fn by<E>(mut self, source: E) -> Self
    where
        E: std::error::Error + Sync + Send + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.msg.is_empty() && self.source.is_some() {
            self.source.as_ref().unwrap().fmt(f)
        } else {
            write!(f, "{}", self.msg)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|s| s.as_ref() as &(dyn std::error::Error))
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Error::new("parsing failed").by(e)
    }
}

impl From<oal_model::errors::Error> for Error {
    fn from(e: oal_model::errors::Error) -> Self {
        Error::new("").by(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
