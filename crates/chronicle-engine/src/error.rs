use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("invalid IR: {0}")]
    InvalidIr(String),
    #[error("command {0} is not valid in mode {1}")]
    WrongMode(&'static str, String),
    #[error("unknown choice {0}")]
    UnknownChoice(String),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
