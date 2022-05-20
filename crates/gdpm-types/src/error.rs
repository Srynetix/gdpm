use thiserror::Error;

/// Type error
#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Wrong version: {0}")]
    WrongVersion(String),
    #[error("Wrong version kind: {0}")]
    WrongVersionKind(String),
}
