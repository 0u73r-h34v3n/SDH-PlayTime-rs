use thiserror::Error;

/// Main error type for PlayTime operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Time tracking error: {0}")]
    TimeTracking(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convenient Result type alias
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Check if error is a not-found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::NotFound(_))
    }
}
