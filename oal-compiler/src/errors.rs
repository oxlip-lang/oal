#[derive(Debug, Clone)]
pub struct EvalError {
    msg: String,
}

impl EvalError {
    fn new(msg: &str) -> EvalError {
        EvalError { msg: msg.into() }
    }
}

impl From<&str> for EvalError {
    fn from(msg: &str) -> Self {
        Self::new(msg)
    }
}

pub type Result<T> = std::result::Result<T, EvalError>;
