use thiserror::Error;

#[derive(Debug, Error)]
pub enum CntrlError {
    #[error("Configuration Error: {0}")]
    Config(String),

    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("AI Error: {0}")]
    Ai(String),

    #[error("Browser Engine Error: {0}")]
    Browser(String),

    #[error("Keychain Error: {0}")]
    Keychain(String),

    #[error("Database Error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Memory Error: {0}")]
    Memory(String),

    #[error("Background Task Error: {0}")]
    Background(#[from] crate::services::background::error::BackgroundError),
}
