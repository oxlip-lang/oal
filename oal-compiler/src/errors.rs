use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Kind {
    Unknown,
    IdentifierNotInScope,
    IdentifierNotAFunction,
    InvalidTypes,
    RelationConflict,
    UnexpectedExpression,
    InvalidYAML,
    CycleDetected,
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

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}: {}\n", self.kind, self.msg)?;
        if !self.details.is_empty() {
            write!(f, "Details:\n")?;
            self.details
                .iter()
                .try_for_each(|d| write!(f, " {}\n", d))?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {}

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Error::new(Kind::InvalidYAML, e.to_string().as_str())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
