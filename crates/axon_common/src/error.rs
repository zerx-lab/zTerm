//! Error types for Axon Terminal

use thiserror::Error;

/// Common error type for Axon Terminal
#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("PTY error: {0}")]
    Pty(String),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("{0}")]
    Other(String),
}

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn config(msg: impl Into<String>) -> Self {
        Error::Config(msg.into())
    }

    pub fn terminal(msg: impl Into<String>) -> Self {
        Error::Terminal(msg.into())
    }

    pub fn pty(msg: impl Into<String>) -> Self {
        Error::Pty(msg.into())
    }

    pub fn platform_not_supported(msg: impl Into<String>) -> Self {
        Error::PlatformNotSupported(msg.into())
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Error::Other(msg.into())
    }
}
