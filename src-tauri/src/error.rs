use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
pub enum AppError {
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convert AppError into a String so it can be returned from Tauri commands.
impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}
