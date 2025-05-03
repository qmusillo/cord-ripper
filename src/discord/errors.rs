use serenity::Error as SerenityError;
use thiserror::Error;

/// Custom error type for the application
#[derive(Debug, Error)]
pub enum DiscordError {
    #[error("HTTP request failed: {0}")]
    HttpRequestFailed(String),

    #[error("Failed to respond to commmand interaction: {0}")]
    CommandInteractionResponseFailed(String),

    #[error("Failed to respond to modal interaction: {0}")]
    MessageInteractionResponseFailed(String),

    #[error("Failed to respond to component interaction: {0}")]
    ComponentInteractionResponseFailed(String),

    #[error("Failed to edit response: {0}")]
    EditResponseFailed(String),

    #[error("Failed to edit message: {0}")]
    EditMessageFailed(String),

    #[error("Failed to send message: {0}")]
    SendMessageFailed(String),

    #[error("Invalid token provided")]
    InvalidToken,

    #[error("Invalid interaction call")]
    InvalidInteractionCall,

    #[error("Invalid Component Data")]
    InvalidComponentData,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Unexpected error occurred: {0}")]
    Unexpected(String),

    #[error("Failed to defer interaction: {0}")]
    DeferFailed(String),

    #[error("Task was cancelled")]
    TaskCancelled,

    #[error("MakeMKV error: {0}")]
    MakeMkvError(#[from] crate::makemkv::errors::MakeMkvError),
}

impl From<SerenityError> for DiscordError {
    fn from(error: SerenityError) -> Self {
        DiscordError::Unexpected(error.to_string())
    }
}

/// Custom result type for convenience
pub type Result<T> = std::result::Result<T, DiscordError>;
