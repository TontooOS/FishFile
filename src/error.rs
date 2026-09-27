/// Result type used throughout fishfile.
pub type Result<T> = std::result::Result<T, FishError>;

/// All errors produced by FishFile.
#[derive(Debug)]
pub enum FishError {
    Io(std::io::Error),
    Parse { line: usize, col: usize, message: String },
    Type { expected: String, found: String },
    KeyNotFound(String),
    InvalidPath(String),
    Json(String),
    Utf8(std::string::FromUtf8Error),
    Custom(String),
}

impl std::fmt::Display for FishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Parse { line, col, message } => {
                write!(f, "parse error at line {}, column {}: {}", line, col, message)
            }
            Self::Type { expected, found } => {
                write!(f, "type error: expected {}, found {}", expected, found)
            }
            Self::KeyNotFound(key) => write!(f, "key not found: {}", key),
            Self::InvalidPath(path) => write!(f, "invalid path: {}", path),
            Self::Json(message) => write!(f, "json error: {}", message),
            Self::Utf8(e) => write!(f, "utf8 error: {}", e),
            Self::Custom(message) => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for FishError {}

impl From<std::io::Error> for FishError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::string::FromUtf8Error> for FishError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::Utf8(e)
    }
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


