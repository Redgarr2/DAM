//! UI-specific error types

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum UiError {
    #[error("Application initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Search failed: {0}")]
    SearchFailed(String),
    
    #[error("File operation failed: {0}")]
    FileOperationFailed(String),
    
    #[error("Import failed: {0}")]
    ImportFailed(String),
    
    #[error("Settings error: {0}")]
    SettingsError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<schema::DamError> for UiError {
    fn from(err: schema::DamError) -> Self {
        UiError::InternalError(err.to_string())
    }
}

impl From<std::io::Error> for UiError {
    fn from(err: std::io::Error) -> Self {
        UiError::FileOperationFailed(err.to_string())
    }
}

impl From<serde_json::Error> for UiError {
    fn from(err: serde_json::Error) -> Self {
        UiError::InternalError(format!("JSON error: {}", err))
    }
}

pub type UiResult<T> = Result<T, UiError>;
