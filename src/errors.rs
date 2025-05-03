use thiserror::Error;

pub type Result<T> = std::result::Result<T, CordRipperError>;

#[derive(Debug, Error)]
pub enum CordRipperError {
    #[error("MakeMkv error: {0}")]
    MakeMkvError(#[from] crate::makemkv::errors::MakeMkvError),

    #[error("Discord error: {0}")]
    DiscordError(#[from] crate::discord::errors::DiscordError),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}
