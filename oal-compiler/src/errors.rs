use std::fmt::Debug;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Kind {
    Unknown,
    IdentifierNotInScope,
    IdentifierNotAFunction,
    CannotUnify,
    RelationConflict,
    UnexpectedExpression,
    InvalidYAML,
}

impl Default for Kind {
    fn default() -> Self {
        Kind::Unknown
    }
}

#[derive(Debug, Clone, Default)]
pub struct Error {
    pub kind: Kind,
    msg: String,
    details: Vec<String>,
}

impl Error {
    pub fn new(kind: Kind, msg: &str) -> Error {
        Error {
            kind,
            msg: msg.into(),
            details: Vec::new(),
        }
    }

    pub fn with<T: Debug>(mut self, e: &T) -> Self {
        self.details.push(format!("{:?}", e));
        self
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Error::new(Kind::InvalidYAML, e.to_string().as_str())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
