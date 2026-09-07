use thiserror::Error;

/// Result type used throughout fishfile.
pub type Result<T> = std::result::Result<T, FishError>;

/// All errors produced by FishFile.
#[derive(Debug, Error)]
pub enum FishError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error at line {line}, column {col}: {message}")]
    Parse { line: usize, col: usize, message: String },

    #[error("type error: expected {expected}, found {found}")]
    Type { expected: String, found: String },

    #[error("key not found: {0}")]
    KeyNotFound(String),

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("serde error: {0}")]
    Serde(String),

    #[error("utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("{0}")]
    Custom(String),
}

impl FishError {
    pub fn parse(line: usize, col: usize, message: impl Into<String>) -> Self {
        Self::Parse {
            line,
            col,
            message: message.into(),
        }
    }

    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }
}

impl From<serde_json::Error> for FishError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e.to_string())
    }
}
