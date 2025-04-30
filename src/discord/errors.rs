use thiserror::Error;

/// Custom error type for the application
#[derive(Debug, Error)]
pub enum DiscordError {
    #[error("HTTP request failed: {0}")]
    HttpRequestFailed(String),

    #[error("Invalid token provided")]
    InvalidToken,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Unexpected error occurred: {0}")]
    Unexpected(String),
}

/// Custom result type for convenience
pub type Result<T> = std::result::Result<T, DiscordError>;
