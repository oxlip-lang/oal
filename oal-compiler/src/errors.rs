#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
}

impl Error {
    pub fn new(msg: &str) -> Error {
        Error { msg: msg.into() }
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self::new(msg)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
