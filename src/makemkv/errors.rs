use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, MakeMkvError>;

#[derive(Debug, Error)]
pub enum MakeMkvError {
    #[error("Failed to execute MakeMKV command: {0}")]
    CommandExecutionError(String),

    #[error("Invalid output format: {0}")]
    InvalidOutputFormat(String),

    #[error("File not found: {0}")]
    FileNotFoundError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Unknown error occurred")]
    UnknownError,

    #[error("Failed to lock drive")]
    LockError,

    #[error("Drive in use: {0}")]
    DriveInUseError(u8),

    #[error("Failed to create temporary directory")]
    TempDirError,

    #[error("Failed to save disc")]
    FailedToSaveDisc,

    #[error("Drive failed to save disc due to drive error")]
    DriveError,

    #[error("Failed to create output directory")]
    OutputDirError,

    #[error("Could not find any drives")]
    NoDrivesFound,

    #[error("Failed to parse MakeMKV output: {0}")]
    ParseError(String),
}

// Example usage
impl MakeMkvError {
    pub fn log_error(&self) {
        eprintln!("Error: {}", self);
    }
}

impl From<io::Error> for MakeMkvError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => MakeMkvError::FileNotFoundError(error.to_string()),
            io::ErrorKind::PermissionDenied => MakeMkvError::PermissionDenied(error.to_string()),
            _ => MakeMkvError::UnknownError,
        }
    }
}

impl From<std::string::FromUtf8Error> for MakeMkvError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        MakeMkvError::InvalidOutputFormat(error.to_string())
    }
}

impl From<std::num::ParseIntError> for MakeMkvError {
    fn from(error: std::num::ParseIntError) -> Self {
        MakeMkvError::InvalidOutputFormat(error.to_string())
    }
}
