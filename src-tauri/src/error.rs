//! Errors returned to the frontend.

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("the tracking service thread is not responding")]
    ServiceUnavailable,
    #[error("{0}")]
    Source(String),
    #[error("could not save the settings: {0}")]
    Settings(String),
    #[error("could not write the file: {0}")]
    Io(#[from] std::io::Error),
    #[error("could not use the clipboard: {0}")]
    Clipboard(String),
}

// The frontend receives the message as a plain string.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
